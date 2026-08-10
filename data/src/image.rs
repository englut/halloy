use std::path::PathBuf;

use iced_gif::widget::gif;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::cache::HexDigest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Format {
    #[serde(with = "serde_image_format")]
    Raster(image::ImageFormat),
    Svg,
}

impl Format {
    pub fn from_magic_bytes(bytes: &[u8]) -> Option<Format> {
        image::guess_format(bytes).ok().map(Format::Raster)
    }

    pub fn from_mime_type(mime_type: &str) -> Option<Format> {
        match mime_type {
            "image/svg+xml" | "image/svg+xml; charset=utf-8" => {
                Some(Format::Svg)
            }
            _ => image::ImageFormat::from_mime_type(mime_type)
                .map(Format::Raster),
        }
    }

    pub fn to_mime_type(&self) -> &'static str {
        match self {
            Format::Raster(format) => format.to_mime_type(),
            Format::Svg => "image/svg+xml",
        }
    }

    pub fn extensions_str(&self) -> &'static [&'static str] {
        match self {
            Format::Raster(format) => format.extensions_str(),
            Format::Svg => &["svg"],
        }
    }
}

pub type Error = image::ImageError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image<'a> {
    pub format: Format,
    pub url: Url,
    pub digest: HexDigest,
    pub path: PathBuf,
    #[serde(skip)]
    pub frames: Option<&'a gif::Frames>,
}

impl<'a> Image<'a> {
    pub fn new(
        format: Format,
        url: Url,
        digest: HexDigest,
        path: PathBuf,
        frames: Option<&'a gif::Frames>,
    ) -> Self {
        Self {
            format,
            url,
            digest,
            path,
            frames,
        }
    }
}

mod serde_image_format {
    use image::ImageFormat;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        format: &ImageFormat,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        format.to_mime_type().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<ImageFormat, D::Error> {
        let s = String::deserialize(deserializer)?;

        ImageFormat::from_mime_type(s)
            .ok_or(serde::de::Error::custom("invalid mime type"))
    }
}
