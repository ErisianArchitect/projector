use std::{fs::File, io::Write, path::{
    Path, PathBuf,
}};

use directories::ProjectDirs;
use eframe::{
    egui::{self, *},
};
use tempfile::NamedTempFile;

use crate::settings::Settings;
use crate::uiext::UiExt;

pub struct SettingsSaver {
    path: PathBuf,
}

impl SettingsSaver {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into()
        }
    }

    pub fn save(&self, settings: &Settings) -> crate::error::Result<()> {
        // save settings to a tempfile first, then upon successfully writing to tempfile, replace
        // the settings file with the tempfile
        let mut temp = tempfile::NamedTempFile::new()?;
        // let file = std::fs::File::create(temp_path.path())?;
        {
            let mut bufwrite = std::io::BufWriter::new(&mut temp);
            bincode::encode_into_std_write(settings, &mut bufwrite, bincode::config::standard())?;
            bufwrite.flush()?;
        }

        temp.as_file_mut().sync_all()?;

        temp.persist(&self.path)?;

        Ok(())
    }
}

pub struct AppConfig {
    path: PathBuf,
}

impl AppConfig {
    const SETTINGS_NAME: &'static str = ".settings";
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
        }
    }

    pub fn from(qualifier: &str, organization: &str, application: &str) -> Self {
        let dirs = directories::ProjectDirs::from(qualifier, organization, application).expect("Failed to create project dirs.");
        Self::new(dirs.config_dir())
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[inline]
    pub fn relative_path<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf {
        self.path.join(relative_path.as_ref())
    }

    pub fn create<P: AsRef<Path>>(&self, relative_path: P) -> std::io::Result<File> {
        File::create(self.relative_path(relative_path))
    }

    pub fn create_new<P: AsRef<Path>>(&self, relative_path: P) -> std::io::Result<File> {
        File::create_new(self.relative_path(relative_path))
    }

    pub fn open<P: AsRef<Path>>(&self, relative_path: P) -> std::io::Result<File> {
        File::open(self.relative_path(relative_path))
    }

    pub fn temp_file(&self) -> std::io::Result<File> {
        tempfile::tempfile_in(self.path())
    }

    pub fn named_temp_file(&self) -> std::io::Result<NamedTempFile> {
        tempfile::NamedTempFile::new_in(self.path())
    }

    pub fn save<P: AsRef<Path>, T: bincode::Encode>(&self, relative_path: P, value: &T) -> crate::error::Result<()> {
        let mut temp = self.named_temp_file()?;
        let save_path = self.relative_path(relative_path);

        {
            let mut bufwrite = std::io::BufWriter::new(&mut temp);
            bincode::encode_into_std_write(value, &mut bufwrite, bincode::config::standard())?;
            bufwrite.flush()?;
        }

        temp.as_file().sync_all()?;

        temp.persist(save_path.as_path())?;

        Ok(())
    }

    pub fn load<P: AsRef<Path>, T: bincode::Decode<()>>(&self, relative_path: P) -> crate::error::Result<T> {
        // let load_path = self.relative_path(relative_path);
        let file = self.open(relative_path)?;
        let mut reader = std::io::BufReader::new(file);
        Ok(bincode::decode_from_std_read(&mut reader, bincode::config::standard())?)
    }

    #[inline]
    pub fn save_settings(&self, settings: &Settings) -> crate::error::Result<()> {
        self.save(Self::SETTINGS_NAME, settings)
    }

    #[inline]
    pub fn load_settings(&self) -> crate::error::Result<Settings> {
        self.load(Self::SETTINGS_NAME)
    }

    pub fn delete<P: AsRef<Path>>(&self, relative_path: P) -> std::io::Result<()> {
        std::fs::remove_file(self.relative_path(relative_path))
    }
}

pub struct AppCache {
    path: PathBuf,
}

impl AppCache {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
        }
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[inline]
    pub fn relative_path<P: AsRef<Path>>(&self, relative_path: &P) -> PathBuf {
        self.path.join(relative_path.as_ref())
    }

    pub fn create<P: AsRef<Path>>(&self, relative_path: &P) -> std::io::Result<File> {
        File::create(self.relative_path(relative_path))
    }

    pub fn create_new<P: AsRef<Path>>(&self, relative_path: &P) -> std::io::Result<File> {
        File::create_new(self.relative_path(relative_path))
    }

    pub fn open<P: AsRef<Path>>(&self, relative_path: &P) -> std::io::Result<File> {
        File::open(self.relative_path(relative_path))
    }

    pub fn temp_file(&self) -> std::io::Result<File> {
        tempfile::tempfile_in(self.path())
    }

    pub fn named_temp_file(&self) -> std::io::Result<NamedTempFile> {
        tempfile::NamedTempFile::new_in(self.path())
    }
}

pub struct AppData {
    config: AppConfig,
    cache: AppCache,
    // plugins: PathBuf,
}

impl AppData {
    pub fn from(qualifier: &str, org: &str, app: &str) -> crate::error::Result<Self> {
        let dirs = directories::ProjectDirs::from(qualifier, org, app).ok_or(crate::error::Error::TempErr("ProjectDirs not created."))?;
        let config = AppConfig::new(dirs.config_dir());
        let cache = AppCache::new(dirs.cache_dir());
        Ok(Self {
            config,
            cache,
        })
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.config.path())?;
        std::fs::create_dir_all(self.cache.path())?;
        Ok(())
    }

    #[inline]
    pub fn save_settings(&self, settings: &Settings) -> crate::error::Result<()> {
        self.config.save_settings(settings)
    }

    #[inline]
    pub fn load_settings(&self) -> crate::error::Result<Settings> {
        self.config.load_settings()
    }

    #[inline]
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    #[inline]
    pub fn cache(&self) -> &AppCache {
        &self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn appdata_test() -> crate::error::Result<()> {

        #[derive(Debug, bincode::Encode, bincode::Decode)]
        pub struct Foo {
            name: String,
            age: u8,
        }

        let appdata = AppData::from("com", "erisianarchitect", "test")?;
        appdata.config().save("test", &Foo { name: "Derek".into(), age: 31 })?;
        let foo: Foo = appdata.config().load("test")?;
        println!("Name: {}\n Age: {}", foo.name, foo.age);
        appdata.config().delete("test")?;
        let test_path = appdata.config().relative_path("test");
        assert!(!test_path.exists());
        Ok(())
    }
}