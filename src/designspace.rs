//! Reading and writing designspace files.

#![deny(rustdoc::broken_intra_doc_links)]

use serde::Serialize;
use std::{fs, fs::File, io::BufReader, path::Path};

use plist::Dictionary;

use crate::error::{DesignSpaceLoadError, DesignSpaceSaveError};
use crate::serde_xml_plist as serde_plist;

/// A [designspace].
///
/// [designspace]: https://fonttools.readthedocs.io/en/latest/designspaceLib/index.html
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename = "designspace")]
pub struct DesignSpaceDocument {
    /// Design space format version.
    #[serde(rename = "@format")]
    pub format: f32,
    /// One or more axes.
    #[serde(deserialize_with = "serde_impls::deserialize_axes")]
    pub axes: Vec<Axis>,
    /// One or more sources.
    #[serde(deserialize_with = "serde_impls::deserialize_sources")]
    pub sources: Vec<Source>,
    /// One or more instances.
    #[serde(default, deserialize_with = "serde_impls::deserialize_instances")]
    pub instances: Vec<Instance>,
    /// Additional arbitrary user data
    #[serde(default, deserialize_with = "serde_plist::deserialize_dict")]
    pub lib: Dictionary,
}

/// An [axis].
///
/// [axis]: https://fonttools.readthedocs.io/en/latest/designspaceLib/xml.html#axis-element
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename = "axis")]
pub struct Axis {
    /// Name of the axis that is used in the location elements.
    #[serde(rename = "@name")]
    pub name: String,
    /// 4 letters. Some axis tags are registered in the OpenType Specification.
    #[serde(rename = "@tag")]
    pub tag: String,
    /// The default value for this axis, in user space coordinates.
    #[serde(rename = "@default")]
    pub default: f32,
    /// Records whether this axis needs to be hidden in interfaces.
    #[serde(default)]
    #[serde(rename = "@hidden")]
    pub hidden: bool,
    /// The minimum value for a continuous axis, in user space coordinates.
    #[serde(rename = "@minimum")]
    pub minimum: Option<f32>,
    /// The maximum value for a continuous axis, in user space coordinates.
    #[serde(rename = "@maximum")]
    pub maximum: Option<f32>,
    /// The possible values for a discrete axis, in user space coordinates.
    #[serde(rename = "@values")]
    pub values: Option<Vec<f32>>,
    /// Mapping between user space coordinates and design space coordinates.
    pub map: Option<Vec<AxisMapping>>,
}

/// Maps one input value (user space coord) to one output value (design space coord).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename = "map")]
pub struct AxisMapping {
    /// user space coordinate
    #[serde(rename = "@input")]
    pub input: f32,
    /// designspace coordinate
    #[serde(rename = "@output")]
    pub output: f32,
}

/// A [source].
///
/// [source]: https://fonttools.readthedocs.io/en/latest/designspaceLib/xml.html#id25
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename = "source")]
pub struct Source {
    /// The family name of the source font.
    #[serde(rename = "@familyname")]
    pub familyname: Option<String>,
    /// The style name of the source font.
    #[serde(rename = "@stylename")]
    pub stylename: Option<String>,
    /// A unique name that can be used to identify this font if it needs to be referenced elsewhere.
    #[serde(rename = "@name")]
    pub name: Option<String>,
    /// A path to the source file, relative to the root path of this document.
    ///
    /// The path can be at the same level as the document or lower.
    #[serde(rename = "@filename")]
    pub filename: String,
    /// The name of the layer in the source file.
    ///
    /// If no layer attribute is given assume the foreground layer should be used.
    #[serde(rename = "@layer")]
    pub layer: Option<String>,
    /// Location in designspace coordinates.
    #[serde(deserialize_with = "serde_impls::deserialize_location")]
    pub location: Vec<Dimension>,
}

/// An [instance].
///
/// [instance]: https://fonttools.readthedocs.io/en/latest/designspaceLib/xml.html#instance-element
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename = "instance")]
pub struct Instance {
    // per @anthrotype, contrary to spec, filename, familyname and stylename are optional
    /// The family name of the instance font. Corresponds with font.info.familyName
    #[serde(rename = "@familyname")]
    pub familyname: Option<String>,
    /// The style name of the instance font. Corresponds with font.info.styleName
    #[serde(rename = "@stylename")]
    pub stylename: Option<String>,
    /// A unique name that can be used to identify this font if it needs to be referenced elsewhere.
    #[serde(rename = "@name")]
    pub name: Option<String>,
    /// A path to the instance file, relative to the root path of this document. The path can be at the same level as the document or lower.
    #[serde(rename = "@filename")]
    pub filename: Option<String>,
    /// Corresponds with font.info.postscriptFontName
    #[serde(rename = "@postscriptfontname")]
    pub postscriptfontname: Option<String>,
    /// Corresponds with styleMapFamilyName
    #[serde(rename = "@stylemapfamilyname")]
    pub stylemapfamilyname: Option<String>,
    /// Corresponds with styleMapStyleName
    #[serde(rename = "@stylemapstylename")]
    pub stylemapstylename: Option<String>,
    /// Location in designspace.
    #[serde(deserialize_with = "serde_impls::deserialize_location")]
    pub location: Vec<Dimension>,
    /// Arbitrary data about this instance
    #[serde(default, deserialize_with = "serde_plist::deserialize_dict")]
    pub lib: Dictionary,
}

/// A [design space dimension].
///
/// [design space location]: https://fonttools.readthedocs.io/en/latest/designspaceLib/xml.html#location-element-source
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename = "dimension")]
pub struct Dimension {
    /// Name of the axis, e.g. Weight.
    #[serde(rename = "@name")]
    pub name: String,
    /// Value on the axis in user coordinates.
    #[serde(rename = "@uservalue")]
    pub uservalue: Option<f32>,
    /// Value on the axis in designcoordinates.
    #[serde(rename = "@xvalue")]
    pub xvalue: Option<f32>,
    /// Separate value for anisotropic interpolations.
    #[serde(rename = "@yvalue")]
    pub yvalue: Option<f32>,
}

impl DesignSpaceDocument {
    /// Load a designspace.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<DesignSpaceDocument, DesignSpaceLoadError> {
        let reader = BufReader::new(File::open(path).map_err(DesignSpaceLoadError::Io)?);
        quick_xml::de::from_reader(reader).map_err(DesignSpaceLoadError::DeError)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), DesignSpaceSaveError> {
        let mut buf = String::from("<?xml version='1.0' encoding='UTF-8'?>\n");
        let mut xml_writer = quick_xml::se::Serializer::new(&mut buf);
        xml_writer.indent(' ', 2);
        self.serialize(xml_writer)?;
        buf.push('\n'); // trailing newline
        fs::write(path, buf)?;
        Ok(())
    }
}

mod serde_impls {

    use super::{Axis, Dimension, Instance, Source};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize_location<'de, D>(deserializer: D) -> Result<Vec<Dimension>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            dimension: Vec<Dimension>,
        }
        Helper::deserialize(deserializer).map(|x| x.dimension)
    }

    pub fn deserialize_instances<'de, D>(deserializer: D) -> Result<Vec<Instance>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            instance: Vec<Instance>,
        }
        Helper::deserialize(deserializer).map(|x| x.instance)
    }

    pub fn deserialize_axes<'de, D>(deserializer: D) -> Result<Vec<Axis>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            axis: Vec<Axis>,
        }
        Helper::deserialize(deserializer).map(|x| x.axis)
    }

    pub fn deserialize_sources<'de, D>(deserializer: D) -> Result<Vec<Source>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            source: Vec<Source>,
        }
        Helper::deserialize(deserializer).map(|x| x.source)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use plist::Value;
    use pretty_assertions::assert_eq;

    use crate::designspace::{AxisMapping, Dimension};

    use super::*;

    fn dim_name_xvalue(name: &str, xvalue: f32) -> Dimension {
        Dimension { name: name.to_string(), uservalue: None, xvalue: Some(xvalue), yvalue: None }
    }

    #[test]
    fn read_single_wght() {
        let ds = DesignSpaceDocument::load(Path::new("testdata/single_wght.designspace")).unwrap();
        assert_eq!(1, ds.axes.len());
        let axis = &ds.axes[0];
        assert_eq!(axis.minimum, Some(400.));
        assert_eq!(axis.maximum, Some(600.));
        assert_eq!(axis.default, 500.);
        assert_eq!(
            &vec![AxisMapping { input: 400., output: 100. }],
            ds.axes[0].map.as_ref().unwrap()
        );
        assert_eq!(1, ds.sources.len());
        let weight_100 = dim_name_xvalue("Weight", 100.);
        assert_eq!(vec![weight_100.clone()], ds.sources[0].location);
        assert_eq!(1, ds.instances.len());
        assert_eq!(vec![weight_100], ds.instances[0].location);
    }

    #[test]
    fn read_wght_variable() {
        let ds = DesignSpaceDocument::load("testdata/wght.designspace").unwrap();
        assert_eq!(1, ds.axes.len());
        assert!(ds.axes[0].map.is_none());
        assert_eq!(
            vec![
                ("TestFamily-Regular.ufo".to_string(), vec![dim_name_xvalue("Weight", 400.)]),
                ("TestFamily-Bold.ufo".to_string(), vec![dim_name_xvalue("Weight", 700.)]),
            ],
            ds.sources
                .into_iter()
                .map(|s| (s.filename, s.location))
                .collect::<Vec<(String, Vec<Dimension>)>>()
        );
    }

    // <https://github.com/linebender/norad/issues/300>
    #[test]
    fn load_with_no_instances() {
        DesignSpaceDocument::load("testdata/no_instances.designspace").unwrap();
    }

    #[test]
    fn load_with_no_source_name() {
        let ds = DesignSpaceDocument::load("testdata/optional_source_names.designspace").unwrap();
        assert!(ds.sources[0].name.is_none());
        assert_eq!(ds.sources[1].name.as_deref(), Some("Test Family Bold"));
    }

    #[test]
    fn load_with_no_instance_name() {
        let ds = DesignSpaceDocument::load("testdata/optional_instance_names.designspace").unwrap();
        assert_eq!(ds.instances[0].name.as_deref(), Some("Test Family Regular"));
        assert!(ds.instances[1].name.is_none());
    }

    #[test]
    fn load_lib() {
        let loaded = DesignSpaceDocument::load("testdata/wght.designspace").unwrap();
        assert_eq!(
            loaded.lib.get("org.linebender.hasLoadedLibCorrectly"),
            Some(&Value::String("Absolutely!".into()))
        );

        let params = loaded.instances[0]
            .lib
            .get("com.schriftgestaltung.customParameters")
            .and_then(Value::as_array)
            .unwrap();
        assert_eq!(params[0].as_array().unwrap()[0].as_string(), Some("xHeight"));
        assert_eq!(params[0].as_array().unwrap()[1].as_string(), Some("536"));
        assert_eq!(
            params[1].as_array().unwrap()[1].as_array().unwrap()[0].as_unsigned_integer(),
            Some(2)
        );
    }

    #[test]
    fn load_save_round_trip() {
        // Given
        let dir = tempdir::TempDir::new("norad_designspace_load_save_round_trip").unwrap();
        let mut ds_test_save_location = dir.path().to_path_buf();
        ds_test_save_location.push("wght.designspace");

        // When
        let ds_initial = DesignSpaceDocument::load("testdata/wght.designspace").unwrap();
        ds_initial.save(&ds_test_save_location).expect("failed to save designspace");
        let ds_after = DesignSpaceDocument::load(ds_test_save_location)
            .expect("failed to load saved designspace");

        // Then
        assert_eq!(ds_initial, ds_after);
    }
}
