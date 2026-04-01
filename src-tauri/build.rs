fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut windows_attributes = tauri_build::WindowsAttributes::new();
        // 设置自定义 Manifest。
        // 注意：必须包含 Common-Controls v6 的依赖声明，否则使用 TaskDialog 等原生 UI 组件时会报错“无法定位程序输入点”。
        windows_attributes = windows_attributes.app_manifest(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="highestAvailable" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#,
        );
        let attrs = tauri_build::Attributes::new().windows_attributes(windows_attributes);
        if let Err(e) = tauri_build::try_build(attrs) {
            eprintln!("Failed to build with custom manifest: {}", e);
            tauri_build::build();
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        tauri_build::build();
    }
}
