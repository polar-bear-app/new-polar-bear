use serde::{Deserialize, Serialize};

// NOTE: keep fields alphabetically ordered.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoPlist {
    /// The default language and region for the bundle, as a
    /// language ID.
    #[serde(rename(serialize = "CFBundleDevelopmentRegion"))]
    pub cf_bundle_development_region: Option<String>,
    /// The user-visible name for the bundle, used by Siri and visible
    /// on the iOS Home screen.
    #[serde(rename(serialize = "CFBundleDisplayName"))]
    pub cf_bundle_display_name: Option<String>,
    /// The entry point of the bundle.
    #[serde(rename(serialize = "CFBundleExecutable"))]
    pub cf_bundle_executable: Option<String>,
    /// The icons of the bundle.
    #[serde(rename(serialize = "CFBundleIcons"))]
    pub cf_bundle_icons: Option<CfBundleIcons>,
    /// The icon file of the bundle.
    #[serde(rename(serialize = "CFBundleIconFile"))]
    pub cf_bundle_icon_file: Option<String>,
    /// The icon files of the bundle.
    #[serde(rename(serialize = "CFBundleIconFiles"))]
    #[serde(default)]
    pub cf_bundle_icon_files: Vec<String>,
    /// The icon name of the bundle.
    #[serde(rename(serialize = "CFBundleIconName"))]
    pub cf_bundle_icon_name: Option<String>,
    /// A unique identifier for a bundle.
    #[serde(rename(serialize = "CFBundleIdentifier"))]
    pub cf_bundle_identifier: Option<String>,
    /// The current version of the Information Property List structure.
    #[serde(rename(serialize = "CFBundleInfoDictionaryVersion"))]
    pub cf_bundle_info_dictionary_version: Option<String>,
    /// A user-visible short name for the bundle.
    #[serde(rename(serialize = "CFBundleName"))]
    pub cf_bundle_name: Option<String>,
    /// The type of bundle.
    #[serde(rename(serialize = "CFBundlePackageType"))]
    pub cf_bundle_package_type: Option<String>,
    /// The release or version number of the bundle.
    #[serde(rename(serialize = "CFBundleShortVersionString"))]
    pub cf_bundle_short_version_string: Option<String>,
    /// A replacement for the app name in text-to-speech operations.
    #[serde(rename(serialize = "CFBundleSpokenName"))]
    pub cf_bundle_spoken_name: Option<String>,
    #[serde(rename(serialize = "CFBundleSupportedPlatforms"))]
    #[serde(default)]
    pub cf_bundle_supported_platforms: Option<Vec<String>>,
    /// The version of the build that identifies an iteration of the
    /// bundle.
    #[serde(rename(serialize = "CFBundleVersion"))]
    pub cf_bundle_version: Option<String>,

    #[serde(rename(serialize = "DTCompiler"))]
    pub dt_compiler: Option<String>,
    #[serde(rename(serialize = "DTPlatformBuild"))]
    pub dt_platform_build: Option<String>,
    #[serde(rename(serialize = "DTPlatformName"))]
    pub dt_platform_name: Option<String>,
    #[serde(rename(serialize = "DTPlatformVersion"))]
    pub dt_platform_version: Option<String>,
    #[serde(rename(serialize = "DTSDKBuild"))]
    pub dt_sdk_build: Option<String>,
    #[serde(rename(serialize = "DTSDKName"))]
    pub dt_sdk_name: Option<String>,
    #[serde(rename(serialize = "DTXcode"))]
    pub dt_xcode: Option<String>,
    #[serde(rename(serialize = "DTXcodeBuild"))]
    pub dt_xcode_build: Option<String>,

    /// The category that best describes your app for the App Store.
    #[serde(rename(serialize = "LSApplicationCategoryType"))]
    pub ls_application_category_type: Option<String>,
    /// The minimum version of the operating system required for
    /// the app to run in macOS.
    #[serde(rename(serialize = "LSMinimumSystemVersion"))]
    pub ls_minimum_system_version: Option<String>,
    /// A boolean value indicating whether the app must run in iOS.
    #[serde(rename(serialize = "LSRequiresIPhoneOS"))]
    pub ls_requires_ios: Option<bool>,

    /// The minimum version of the operating system required for
    /// the app to run in iOS, iPadOS, tvOS, and watchOS.
    #[serde(rename(serialize = "MinimumOSVersion"))]
    pub minimum_os_version: Option<String>,

    /// A message that tells the user why the app is requesting
    /// access to the device's camera.
    #[serde(rename(serialize = "NSCameraUsageDescription"))]
    pub ns_camera_usage_description: Option<String>,
    /// A human-readable copyright notice for the bundle.
    #[serde(rename(serialize = "NSHumanReadableCopyright"))]
    pub ns_human_readable_copyright: Option<String>,

    #[serde(rename(serialize = "UIDeviceFamily"))]
    pub ui_device_family: Option<Vec<u64>>,
    #[serde(rename(serialize = "UILaunchScreen"))]
    pub ui_launch_screen: Option<UiLaunchScreen>,
    #[serde(rename(serialize = "UILaunchStoryboardName"))]
    pub ui_launch_storyboard_name: Option<String>,
    #[serde(rename(serialize = "UIRequiredDeviceCapabilities"))]
    pub ui_required_device_capabilities: Option<Vec<String>>,
    #[serde(rename(serialize = "UISupportedInterfaceOrientations~ipad"))]
    #[serde(default)]
    pub ui_supported_interface_orientations_ipad: Vec<String>,
    #[serde(rename(serialize = "UISupportedInterfaceOrientations~iphone"))]
    #[serde(default)]
    pub ui_supported_interface_orientations_iphone: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiLaunchScreen {
    #[serde(rename(serialize = "UIColorName"))]
    pub ui_color_name: Option<String>,
    #[serde(rename(serialize = "UIImageName"))]
    pub ui_image_name: Option<String>,
    #[serde(rename(serialize = "UIImageRespectsSafeAreaInsets"))]
    pub ui_image_respects_safe_area_insets: Option<bool>,
    #[serde(rename(serialize = "UINavigationBar"))]
    pub ui_navigation_bar: Option<bool>,
    #[serde(rename(serialize = "UITabBar"))]
    pub ui_tab_bar: Option<bool>,
    #[serde(rename(serialize = "UIToolbar"))]
    pub ui_toolbar: Option<bool>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CfBundleIcons {
    #[serde(rename(serialize = "CFBundlePrimaryIcon"))]
    pub cf_bundle_primary_icon: Option<CfBundlePrimaryIcon>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CfBundlePrimaryIcon {
    #[serde(rename(serialize = "CFBundleIconName"))]
    pub cf_bundle_icon_name: Option<String>,
}
