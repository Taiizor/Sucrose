## Using WinUIEx with .NET MAUI

### Include WinUIEx package in Windows target

Add WinUIEx to the Windows build target by adding a package reference in the `.csproj` file:
```xml
  <ItemGroup>
    <PackageReference Include="WinUIEx" Version="2.3.0" Condition="$([MSBuild]::GetTargetPlatformIdentifier('$(TargetFramework)')) == 'windows'" />
  </ItemGroup>
```

If the referenced version of WinUIEx relies on a newer version of the Windows App SDK than .NET MAUI implicitly references, you'll likely see errors like this:
```
Error	NU1605	Warning As Error: Detected package downgrade: Microsoft.WindowsAppSDK from 1.4.230822000 to 1.2.221209.1.
Reference the package directly from the project to select a different version. MauiApp8 -> WinUIEx 2.3.0 -> Microsoft.WindowsAppSDK (>= 1.4.230822000) 
```
To address that, explicitly reference the Windows App SDK package with the version mentioned in the error. For example:
```xml
<ItemGroup>
    <PackageReference Include="Microsoft.WindowsAppSDK" Version="1.4.230822000" Condition="$([MSBuild]::GetTargetPlatformIdentifier('$(TargetFramework)')) == 'windows'" />
</ItemGroup>
```

### Use WinUIEx's WebAuthenticator instead of .NET MAUI's
.NET MAUI does not support the WebAuthenticator out of the box on Windows. Instead you can use WinUIEx to get the same capability.
First in `\Platforms\Windows\App.xaml.cs`, add the `WebAuthenticator.CheckOAuthRedirectionActivation` to the App constructor and return if it returns `true`:
```cs
    public App()
    {
        if (WinUIEx.WebAuthenticator.CheckOAuthRedirectionActivation())
            return;
        this.InitializeComponent();
    }
```
When you want to perform an OAuth request, on Windows use WinUIEx's authenticator, instead of .NET MAUI's:
```cs
#if WINDOWS
    var result = await WinUIEx.WebAuthenticator.AuthenticateAsync(new Uri(authUri), new Uri(redirectUri));
#else
    var result = await WebAuthenticator.AuthenticateAsync(new Uri(authUri), new Uri(redirectUri));
#endif
````
Lastly, make sure you've registered the redirect uri-scheme in `Platforms\Windows\Package.appxmanifest`, as described in [WebAuthenticator](WebAuthenticator.md).


### Perform operations when Windows are created

Use the `ConfigureLifecycleEvents` to be notified of new windows getting created, which you can then extend with WinUIEx.

```cs
using Microsoft.Maui.LifecycleEvents;

#if WINDOWS
using WinUIEx;
#endif

namespace MyApp.Maui
{
    public static class MauiProgram
    {
        public static MauiApp CreateMauiApp()
        {
            var builder = MauiApp.CreateBuilder();
            builder
                .UseMauiApp<App>()
                .ConfigureFonts(fonts =>
                {
                    fonts.AddFont("OpenSans-Regular.ttf", "OpenSansRegular");
                    fonts.AddFont("OpenSans-Semibold.ttf", "OpenSansSemibold");
                });

#if WINDOWS
            builder.ConfigureLifecycleEvents(events =>
            {
                events.AddWindows(wndLifeCycleBuilder =>
                {
                    wndLifeCycleBuilder.OnWindowCreated(window =>
                    {
                        window.CenterOnScreen(1024,768); //Set size and center on screen using WinUIEx extension method

                        var manager = WinUIEx.WindowManager.Get(window);
                        manager.PersistenceId = "MainWindowPersistanceId"; // Remember window position and size across runs
                        manager.MinWidth = 640;
                        manager.MinHeight = 480;
                    });
                });
            });
#endif

            return builder.Build();
        }
    }
}
```

### Interact with the Page's containing window:

```cs
namespace MyApp.Maui;
#if WINDOWS
using WinUIEx;
#endif

public partial class MainPage : ContentPage
{
    public MainPage()
    {
        InitializeComponent();
    }

    private void OnMaximizeClicked(object sender, EventArgs e)
    {
#if WINDOWS
        var window = this.Window.Handler.PlatformView as Microsoft.UI.Xaml.Window;
        window.Maximize(); // Use WinUIEx Extension method to maximize window
#endif
    }
    

    private void OnFullScreenClicked(object sender, EventArgs e)
    {
#if WINDOWS
        // Get the window manager
        var manager = WinUIEx.WindowManager.Get(this.Window.Handler.PlatformView as Microsoft.UI.Xaml.Window);
        if (manager.PresenterKind == Microsoft.UI.Windowing.AppWindowPresenterKind.Overlapped)
            manager.PresenterKind = Microsoft.UI.Windowing.AppWindowPresenterKind.FullScreen;
        else
            manager.PresenterKind = Microsoft.UI.Windowing.AppWindowPresenterKind.Overlapped;
#endif
    }
}
```
