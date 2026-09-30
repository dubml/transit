pub mod yamlviajson {
    use serde::{de,ser};

    pub fn from_str<T>(s: &str) -> anyhow::Result<T>
    where
        T: for<'de> de::Deserialize<'de>,
    {
        let de_yaml = serde_yaml::Deserializer::from_str(s);
        let mut buf = Vec::with_capacity(128);
        {
            let mut se_json = serde_json::Serializer::new(&mut buf);
            serde_transcode::transcode(de_yaml, &mut se_json)?;
        }
        Ok(serde_json_path_to_error::from_slice(&buf)?)
    }
    pub fn to_string<T>(value: &T) -> anyhow::Result<String>
    where
        T: ?Sized + ser::Serialize,
    {
        let js = serde_json::to_string(value)?;
        let mut buf = Vec::with_capacity(128);
        {
            let mut se_yaml = serde_yaml::Serializer::new(&mut buf);
            let de_serde = serde_yaml::Deserializer::from_str(&js);
            serde_transcode::transcode(de_serde, &mut se_yaml)?;
        }


        Ok(String::from_utf8(buf)?)
    }
}