use std::{
    alloc::{
        alloc,
        Layout,
    },
    borrow::Cow,
    collections::HashMap,
    hash::Hasher,
    path::{
        Path,
        PathBuf
    },
    ptr::NonNull,
    rc::Rc,
    sync::{
        Arc,
        LazyLock,
        Mutex,
    }
};
use twox_hash::XxHash64;

const HASH_SEED: u64 = 0x9e3779b9;

/// Hash `bytes` with [XxHash64].
#[must_use]
#[inline]
pub fn hash_bytes(bytes: &[u8]) -> u64 {
    XxHash64::oneshot(HASH_SEED, bytes)
}

/// Hash with [XxHash64] `head_size` bytes at the beginning of the buffer
/// and `tail_size` bytes at the end of the buffer (in that order). if the
/// length of the buffer is less than or equal to `head_size + tail_size`,
/// then the full buffer is hashed.
#[must_use]
pub fn hash_bytes_head_tail(bytes: &[u8], head_size: usize, tail_size: usize) -> u64 {
    let ends_total = head_size + tail_size;
    if bytes.len() <= ends_total {
        return hash_bytes(bytes);
    }
    let head = &bytes[0..head_size];
    let tail = &bytes[bytes.len() - tail_size..bytes.len()];
    let mut hasher = XxHash64::with_seed(HASH_SEED);
    hasher.write(head);
    hasher.write(tail);
    hasher.finish()
}

/// Hash with [XxHash64] `end_size` bytes at the beginning of the buffer, and `end_size`
/// bytes at the end of the buffer (in that order). If the buffer size
/// is less than or equal to `end_size + end_size`, then the full buffer
/// is hashed.
#[must_use]
#[inline]
pub fn hash_bytes_ends(bytes: &[u8], end_size: usize) -> u64 {
    hash_bytes_head_tail(bytes, end_size, end_size)
}

/// Hash `string` using [XxHash64].
#[must_use]
#[inline]
pub fn hash_str(string: &str) -> u64 {
    hash_bytes(string.as_bytes())
}

/// Hash with [XxHash64] `head_size` bytes at the beginning of the string
/// and `tail_size` bytes at the end of the string (in that order). if the
/// length of the string is less than or equal to `head_size + tail_size`,
/// then the full string is hashed.
#[must_use]
#[inline]
pub fn hash_str_head_tail(string: &str, head_size: usize, tail_size: usize) -> u64 {
    hash_bytes_head_tail(string.as_bytes(), head_size, tail_size)
}

/// Hash with [XxHash64] `end_size` bytes at the beginning of the string, and `end_size`
/// bytes at the end of the string (in that order). If the string size
/// is less than or equal to `end_size + end_size`, then the full string
/// is hashed.
#[must_use]
#[inline]
pub fn hash_str_ends(string: &str, end_size: usize) -> u64 {
    hash_bytes_ends(string.as_bytes(), end_size)
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtomKey {
    hash: u64,
    len: usize,
}

impl AtomKey {
    #[must_use]
    #[inline]
    pub fn from_str(source: &str) -> AtomKey {
        let hash = hash_str_ends(source, 64);
        let len = source.len();
        AtomKey {
            hash,
            len,
        }
    }
}

#[repr(C)]
struct AtomInner<T: ?Sized> {
    key: AtomKey,
    value: T,
}

impl AtomInner<()> {
    fn fatten(ptr: NonNull<AtomInner<()>>, len: usize) -> NonNull<AtomInner<str>> {
        unsafe {
            let str_ptr = std::ptr::slice_from_raw_parts(ptr.as_ptr(), len) as *mut AtomInner<str>;
            NonNull::new_unchecked(str_ptr)
        }
    }

    fn layout(len: usize) -> Layout {
        Layout::new::<AtomInner<()>>()
            .extend(
                Layout::array::<u8>(len)
                    .unwrap()
            )
            .unwrap()
            .0
            .pad_to_align()
    }

    fn alloc(len: usize) -> Option<NonNull<AtomInner<()>>> {
        let layout = Self::layout(len);
        unsafe {
            let ptr = alloc(layout);
            NonNull::new(ptr as *mut AtomInner<()>)
        }
    }

    fn alloc_new(string: &str, key: AtomKey) -> Option<NonNull<AtomInner<()>>> {
        let ptr = Self::alloc(string.len())?;
        unsafe {
            ptr.write(AtomInner {
                key,
                value: (),
            });
        }
        let mut fat_ptr = Self::fatten(ptr, string.len());
        unsafe {
            std::ptr::copy_nonoverlapping(string.as_ptr() as *mut u8, fat_ptr.as_mut().value.as_mut_ptr() as *mut u8, string.len());
        }
        Some(ptr)
    }
}

unsafe impl Send for AtomInner<()>
where str: Send {}
unsafe impl Sync for AtomInner<()>
where str: Sync {}

/// An [Atom] is a singleton reference to a `'static` lifetime string.
/// The string lives until the end of the program, and its memory is
/// essentially considered "leaked" during execution.
/// 
/// There is no way to deallocate an [Atom] safely since they are cheaply
/// copyable with no reference counting whatsoever. That means that you
/// should be conscientious about how many [Atom] instances you create
/// during the lifetime of your program. Atoms are not meant to be used
/// in place of [String].
#[derive(Clone, Copy)]
pub struct Atom {
    inner: NonNull<AtomInner<()>>,
}

unsafe impl Send for Atom
where AtomInner<()>: Send {}
unsafe impl Sync for Atom
where AtomInner<()>: Sync {}

static INTERN_SET: LazyLock<Mutex<HashMap<AtomKey, Vec<Atom>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

impl Atom {
    #[must_use]
    #[inline]
    fn new_internal(string: &str, key: AtomKey) -> Self {
        let inner = AtomInner::alloc_new(string, key).expect("Out of memory or something.");
        Self {
            inner,
        }
    }

    #[must_use]
    pub fn new(string: &str) -> Self {
        let key = AtomKey::from_str(string);
        let mut set_lock = INTERN_SET.lock().unwrap();
        let atoms = set_lock.entry(key).or_insert_with(|| Vec::new());
        for atom in atoms.iter().cloned() {
            let atom_str = atom.as_str();
            if atom_str == string {
                return atom;
            }
        }
        let atom = Atom::new_internal(string, key);
        atoms.push(atom);
        atom
    }

    #[must_use]
    #[inline]
    pub fn hash(&self) -> u64 {
        unsafe {
            self.inner.as_ref().key.hash
        }
    }

    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        unsafe {
            self.inner.as_ref().key.len
        }
    }

    #[must_use]
    #[inline]
    pub fn as_str(self) -> &'static str {
        unsafe {
            let inner_ref = self.inner.as_ref();
            let len = inner_ref.key.len;
            let str_ptr = std::ptr::slice_from_raw_parts(inner_ref, len) as *mut AtomInner<str>;
            &str_ptr.as_ref().unwrap().value
        }
    }

    #[must_use]
    #[inline]
    pub fn as_path(self) -> &'static Path {
        self.as_str().as_ref()
    }

    #[must_use]
    #[inline]
    pub fn ptr_eq(lhs: &Self, rhs: &Self) -> bool {
        std::ptr::eq(lhs.inner.as_ptr(), rhs.inner.as_ptr())
    }

    #[must_use]
    #[inline]
    pub fn create_string(self) -> String {
        String::from(self)
    }
}

impl std::cmp::PartialEq<Atom> for Atom {
    fn eq(&self, other: &Atom) -> bool {
        // This works because Atoms with the same value
        // will always have the same pointer.
        Atom::ptr_eq(self, other)
    }

    fn ne(&self, other: &Atom) -> bool {
        !Atom::ptr_eq(self, other)
    }
}

impl std::cmp::Eq for Atom {}

impl std::cmp::PartialOrd<Atom> for Atom {
    fn partial_cmp(&self, other: &Atom) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }

    fn ge(&self, other: &Atom) -> bool {
        self.as_str().ge(other.as_str())
    }

    fn gt(&self, other: &Atom) -> bool {
        self.as_str().gt(other.as_str())
    }

    fn le(&self, other: &Atom) -> bool {
        self.as_str().le(other.as_str())
    }

    fn lt(&self, other: &Atom) -> bool {
        self.as_str().lt(other.as_str())
    }
}

impl std::cmp::Ord for Atom {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

// PartialEq str
impl std::cmp::PartialEq<str> for Atom {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }

    fn ne(&self, other: &str) -> bool {
        self.as_str().ne(other)
    }
}

impl std::cmp::PartialEq<Atom> for str {
    fn eq(&self, other: &Atom) -> bool {
        self.eq(other.as_str())
    }

    fn ne(&self, other: &Atom) -> bool {
        self.ne(other.as_str())
    }
}

// PartialOrd str
impl std::cmp::PartialOrd<str> for Atom {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other)
    }

    fn ge(&self, other: &str) -> bool {
        self.as_str().ge(other)
    }

    fn gt(&self, other: &str) -> bool {
        self.as_str().gt(other)
    }

    fn le(&self, other: &str) -> bool {
        self.as_str().le(other)
    }

    fn lt(&self, other: &str) -> bool {
        self.as_str().lt(other)
    }
}

impl std::cmp::PartialOrd<Atom> for str {
    fn partial_cmp(&self, other: &Atom) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }

    fn ge(&self, other: &Atom) -> bool {
        self.ge(other.as_str())
    }

    fn gt(&self, other: &Atom) -> bool {
        self.gt(other.as_str())
    }

    fn le(&self, other: &Atom) -> bool {
        self.le(other.as_str())
    }

    fn lt(&self, other: &Atom) -> bool {
        self.lt(other.as_str())
    }
}

// PartialEq &str
impl std::cmp::PartialEq<&str> for Atom {
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(*other)
    }

    fn ne(&self, other: &&str) -> bool {
        self.as_str().ne(*other)
    }
}

impl std::cmp::PartialEq<Atom> for &str {
    fn eq(&self, other: &Atom) -> bool {
        (*self).eq(other.as_str())
    }

    fn ne(&self, other: &Atom) -> bool {
        (*self).ne(other.as_str())
    }
}

// PartialOrd &str
impl std::cmp::PartialOrd<&str> for Atom {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(*other)
    }

    fn ge(&self, other: &&str) -> bool {
        self.as_str().ge(*other)
    }

    fn gt(&self, other: &&str) -> bool {
        self.as_str().gt(*other)
    }

    fn le(&self, other: &&str) -> bool {
        self.as_str().le(*other)
    }

    fn lt(&self, other: &&str) -> bool {
        self.as_str().lt(*other)
    }
}

impl std::cmp::PartialOrd<Atom> for &str {
    fn partial_cmp(&self, other: &Atom) -> Option<std::cmp::Ordering> {
        (*self).partial_cmp(other.as_str())
    }

    fn ge(&self, other: &Atom) -> bool {
        (*self).ge(other.as_str())
    }

    fn gt(&self, other: &Atom) -> bool {
        (*self).gt(other.as_str())
    }

    fn le(&self, other: &Atom) -> bool {
        (*self).le(other.as_str())
    }

    fn lt(&self, other: &Atom) -> bool {
        (*self).lt(other.as_str())
    }
}

// PartialEq String
impl PartialEq<String> for Atom {
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(other)
    }

    fn ne(&self, other: &String) -> bool {
        self.as_str().ne(other)
    }
}

impl PartialEq<Atom> for String {
    fn eq(&self, other: &Atom) -> bool {
        self.eq(other.as_str())
    }

    fn ne(&self, other: &Atom) -> bool {
        self.ne(other.as_str())
    }
}

// PartialOrd String
impl PartialOrd<String> for Atom {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }

    fn ge(&self, other: &String) -> bool {
        self.ge(other.as_str())
    }

    fn gt(&self, other: &String) -> bool {
        self.gt(other.as_str())
    }

    fn le(&self, other: &String) -> bool {
        self.le(other.as_str())
    }

    fn lt(&self, other: &String) -> bool {
        self.lt(other.as_str())
    }
}

impl PartialOrd<Atom> for String {
    fn partial_cmp(&self, other: &Atom) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }

    fn ge(&self, other: &Atom) -> bool {
        self.as_str().eq(other.as_str())
    }

    fn gt(&self, other: &Atom) -> bool {
        self.as_str().gt(other.as_str())
    }

    fn le(&self, other: &Atom) -> bool {
        self.as_str().le(other.as_str())
    }

    fn lt(&self, other: &Atom) -> bool {
        self.as_str().lt(other.as_str())
    }
}

impl std::ops::Deref for Atom {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Atom {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for Atom {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl From<Atom> for String {
    #[inline]
    fn from(value: Atom) -> Self {
        value.as_str().to_owned()
    }
}

impl From<Atom> for Cow<'static, str> {
    #[inline]
    fn from(value: Atom) -> Self {
        Cow::Borrowed(value.as_str())
    }
}

impl From<Atom> for Box<str> {
    #[inline]
    fn from(value: Atom) -> Self {
        Box::from(value.as_str())
    }
}

impl From<Atom> for Rc<str> {
    #[inline]
    fn from(value: Atom) -> Self {
        Rc::from(value.as_str())
    }
}

impl From<Atom> for Arc<str> {
    #[inline]
    fn from(value: Atom) -> Self {
        Arc::from(value.as_str())
    }
}

impl From<Atom> for Vec<u8> {
    #[inline]
    fn from(value: Atom) -> Self {
        Self::from(value.as_bytes())
    }
}

impl From<Atom> for Vec<char> {
    #[inline]
    fn from(value: Atom) -> Self {
        Self::from_iter(value.chars())
    }
}

impl From<Atom> for &'static str {
    #[inline]
    fn from(value: Atom) -> Self {
        value.as_str()
    }
}

impl From<Atom> for PathBuf {
    #[inline]
    fn from(value: Atom) -> Self {
        PathBuf::from(value.as_str())
    }
}

impl From<&str> for Atom {
    #[inline]
    fn from(value: &str) -> Self {
        Atom::new(value)
    }
}

impl From<String> for Atom {
    #[inline]
    fn from(value: String) -> Self {
        Atom::new(value.as_str())
    }
}

impl From<Box<str>> for Atom {
    #[inline]
    fn from(value: Box<str>) -> Self {
        Atom::new(&value)
    }
}

impl From<Rc<str>> for Atom {
    #[inline]
    fn from(value: Rc<str>) -> Self {
        Atom::new(&value)
    }
}

impl From<Arc<str>> for Atom {
    #[inline]
    fn from(value: Arc<str>) -> Self {
        Atom::new(&value)
    }
}

impl<'a> From<Cow<'a, str>> for Atom {
    #[inline]
    fn from(value: Cow<'a, str>) -> Self {
        Atom::new(&value)
    }
}

impl std::fmt::Display for Atom {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for Atom {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl std::hash::Hash for Atom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            // The key is deterministically derived from the
            // immutable string, so we can just hash the key
            // for fast hashing of Atom types.
            self.inner.as_ref().key.hash(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use hashbrown::HashTable;

    use super::*;
    #[test]
    fn hashmap_test() {
        let mut map = HashMap::new();
        let hello = Atom::new("hello");
        let atom2 = Atom::new("hello");
        assert_eq!(hello, atom2);
        map.insert(hello, "hello, world");
        if let Some(&hello2) = map.get(&Atom::new("hello")) {
            println!("Value: {hello2:?}");
        } else {
            println!("Not found.");
        }
    }

    #[test]
    fn hashtable_test() {
        let mut table = HashTable::<Atom>::new();
        let atom1 = Atom::new("Hello, world");
        let atom2 = Atom::new("this is a test.");
        // let entry = table.entry(atom1.hash(), , Atom::hash)
        //     .or_insert(atom1);
        // let entry_atom = entry.get().clone();
    }
}