use anyhow::Result;
use serde::ser::{SerializeTuple, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "Package")]
pub struct AppxManifest {
    #[serde(rename(serialize = "xmlns"))]
    #[serde(default = "default_namespace")]
    ns: String,
    #[serde(rename(serialize = "xmlns:uap"))]
    #[serde(default = "default_uap_namespace")]
    ns_uap: String,
    #[serde(rename(serialize = "xmlns:rescap"))]
    #[serde(default = "default_rescap_namespace")]
    ns_rescap: String,
    #[serde(rename(serialize = "Identity"))]
    pub identity: Identity,
    #[serde(rename(serialize = "Properties"))]
    pub properties: Properties,
    #[serde(rename(serialize = "Resources"))]
    pub resources: Resources,
    #[serde(rename(serialize = "Dependencies"))]
    pub dependencies: Dependencies,
    #[serde(rename(serialize = "Capabilities"))]
    #[serde(serialize_with = "serialize_element")]
    pub capabilities: Vec<Capability>,
    #[serde(rename(serialize = "Applications"))]
    pub applications: Applications,
}

impl Default for AppxManifest {
    fn default() -> Self {
        Self {
            ns: default_namespace(),
            ns_uap: default_uap_namespace(),
            ns_rescap: default_rescap_namespace(),
            identity: Default::default(),
            properties: Default::default(),
            resources: Default::default(),
            dependencies: Default::default(),
            capabilities: Default::default(),
            applications: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Applications {
    #[serde(rename(serialize = "Application"))]
    pub application: Vec<Application>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Resources {
    #[serde(rename(serialize = "Resource"))]
    pub resource: Vec<Resource>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Dependencies {
    #[serde(rename(serialize = "TargetDeviceFamily"))]
    pub target_device_family: Vec<TargetDeviceFamily>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Identity {
    #[serde(rename(serialize = "Name"))]
    pub name: Option<String>,
    #[serde(rename(serialize = "Version"))]
    pub version: Option<String>,
    #[serde(rename(serialize = "Publisher"))]
    pub publisher: Option<String>,
    #[serde(rename(serialize = "ProcessorArchitecture"))]
    pub processor_architecture: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Properties {
    #[serde(rename(serialize = "DisplayName"))]
    #[serde(serialize_with = "serialize_element")]
    pub display_name: Option<String>,
    #[serde(rename(serialize = "PublisherDisplayName"))]
    #[serde(serialize_with = "serialize_element")]
    pub publisher_display_name: Option<String>,
    #[serde(rename(serialize = "Logo"))]
    #[serde(serialize_with = "serialize_element")]
    pub logo: Option<String>,
    #[serde(rename(serialize = "Description"))]
    #[serde(serialize_with = "serialize_element")]
    pub description: Option<String>,
}

fn serialize_element<S>(value: &impl Serialize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut tuple = serializer.serialize_tuple(1)?;
    tuple.serialize_element(value)?;
    tuple.end()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Resource {
    #[serde(rename(serialize = "Language"))]
    pub language: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TargetDeviceFamily {
    #[serde(rename(serialize = "Name"))]
    pub name: String,
    #[serde(rename(serialize = "MinVersion"))]
    pub min_version: String,
    #[serde(rename(serialize = "MaxVersionTested"))]
    pub max_version: String,
}

impl Default for TargetDeviceFamily {
    fn default() -> Self {
        Self {
            name: "Windows.Desktop".into(),
            min_version: "10.0.0.0".into(),
            max_version: "10.0.20348.0".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Capability {
    #[serde(rename(deserialize = "capability"))]
    #[serde(rename(serialize = "Capability"))]
    Capability {
        #[serde(rename(serialize = "Name"))]
        name: String,
    },
    #[serde(rename(deserialize = "restricted"))]
    #[serde(rename(serialize = "rescap:Capability"))]
    Restricted {
        #[serde(rename(serialize = "Name"))]
        name: String,
    },
    #[serde(rename(deserialize = "device"))]
    #[serde(rename(serialize = "DeviceCapability"))]
    Device {
        #[serde(rename(serialize = "Name"))]
        name: String,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Application {
    #[serde(rename(serialize = "Id"))]
    pub id: Option<String>,
    #[serde(rename(serialize = "Executable"))]
    pub executable: Option<String>,
    #[serde(rename(serialize = "EntryPoint"))]
    pub entry_point: Option<String>,
    #[serde(rename(serialize = "uap:VisualElements"))]
    pub visual_elements: VisualElements,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VisualElements {
    #[serde(rename(serialize = "BackgroundColor"))]
    pub background_color: Option<String>,
    #[serde(rename(serialize = "DisplayName"))]
    pub display_name: Option<String>,
    #[serde(rename(serialize = "Description"))]
    pub description: Option<String>,
    #[serde(rename(serialize = "Square150x150Logo"))]
    pub logo_150x150: Option<String>,
    #[serde(rename(serialize = "Square44x44Logo"))]
    pub logo_44x44: Option<String>,
    #[serde(rename(serialize = "uap:DefaultTile"))]
    pub default_tile: Option<DefaultTile>,
    #[serde(rename(serialize = "uap:SplashScreen"))]
    pub splash_screen: Option<SplashScreen>,
    #[serde(rename(serialize = "uap:LockScreen"))]
    pub lock_screen: Option<LockScreen>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DefaultTile {
    #[serde(rename(serialize = "ShortName"))]
    pub short_name: Option<String>,
    #[serde(rename(serialize = "Square71x71Logo"))]
    pub logo_71x71: Option<String>,
    #[serde(rename(serialize = "Square310x310Logo"))]
    pub logo_310x310: Option<String>,
    #[serde(rename(serialize = "Wide310x150Logo"))]
    pub logo_310x150: Option<String>,
    #[serde(rename(serialize = "uap:ShowNameOnTiles"))]
    pub show_names_on_tiles: ShowNameOnTiles,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ShowNameOnTiles {
    #[serde(rename(serialize = "uap:ShowOn"))]
    pub show_on: Vec<ShowOn>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ShowOn {
    #[serde(rename(serialize = "Tile"))]
    pub tile: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SplashScreen {
    #[serde(rename(serialize = "Image"))]
    pub image: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LockScreen {
    #[serde(rename(serialize = "BadgeLogo"))]
    pub badge_logo: String,
    #[serde(rename(serialize = "Notification"))]
    pub notification: String,
}

fn default_namespace() -> String {
    "http://schemas.microsoft.com/appx/manifest/foundation/windows10".to_string()
}

fn default_uap_namespace() -> String {
    "http://schemas.microsoft.com/appx/manifest/uap/windows10".to_string()
}

fn default_rescap_namespace() -> String {
    "http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_properties() {
        let props = Properties {
            display_name: Some("".into()),
            publisher_display_name: Some("".into()),
            logo: Some("".into()),
            description: Some("".into()),
        };
        let xml = quick_xml::se::to_string(&props).unwrap();
        assert_eq!(xml, "<Properties><DisplayName></DisplayName><PublisherDisplayName></PublisherDisplayName><Logo></Logo><Description></Description></Properties>");
    }

    #[test]
    fn test_manifest() {
        let manifest = AppxManifest {
            identity: Identity {
                name: Some("com.flutter.fluttertodoapp".into()),
                version: Some("1.0.0.0".into()),
                publisher: Some(
                    "CN=Msix Testing, O=Msix Testing Corporation, S=Some-State, C=US".into(),
                ),
                processor_architecture: Some("x64".into()),
            },
            properties: Properties {
                display_name: Some("fluttertodoapp".into()),
                publisher_display_name: Some("com.flutter.fluttertodoapp".into()),
                logo: Some("Images\\StoreLogo.png".into()),
                description: Some("A new Flutter project.".into()),
            },
            resources: Resources {
                resource: vec![Resource {
                    language: "en".into(),
                }],
            },
            dependencies: Dependencies {
                target_device_family: vec![Default::default()],
            },
            capabilities: vec![
                Capability::Capability {
                    name: "internetClient".into(),
                },
                Capability::Restricted {
                    name: "runFullTrust".into(),
                },
                Capability::Device {
                    name: "location".into(),
                },
            ],
            applications: Applications {
                application: vec![Application {
                    id: Some("fluttertodoapp".into()),
                    executable: Some("todoapp.exe".into()),
                    entry_point: Some("Windows.FullTrustApplication".into()),
                    visual_elements: VisualElements {
                        background_color: Some("transparent".into()),
                        display_name: Some("fluttertodoapp".into()),
                        description: Some("A new flutter project.".into()),
                        logo_44x44: Some("Images\\Square44x44Logo.png".into()),
                        logo_150x150: Some("Images\\Square150x150Logo.png".into()),
                        default_tile: Some(DefaultTile {
                            short_name: Some("fluttertodoapp".into()),
                            logo_71x71: Some("Images\\SmallTile.png".into()),
                            logo_310x310: Some("Images\\LargeTile.png".into()),
                            logo_310x150: Some("Images\\Wide310x150Logo.png".into()),
                            show_names_on_tiles: ShowNameOnTiles {
                                show_on: vec![
                                    ShowOn {
                                        tile: "square150x150Logo".into(),
                                    },
                                    ShowOn {
                                        tile: "square310x310Logo".into(),
                                    },
                                    ShowOn {
                                        tile: "wide310x150Logo".into(),
                                    },
                                ],
                            },
                        }),
                        splash_screen: Some(SplashScreen {
                            image: "Images\\SplashScreen.png".into(),
                        }),
                        lock_screen: Some(LockScreen {
                            badge_logo: "Images\\BadgeLogo.png".into(),
                            notification: "badge".into(),
                        }),
                    },
                }],
            },
            ..Default::default()
        };
        let xml = quick_xml::se::to_string(&manifest).unwrap();
        println!("{}", xml);
    }
}
