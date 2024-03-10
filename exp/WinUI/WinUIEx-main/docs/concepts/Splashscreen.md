## Splash Screen

To create a new splash screen, first create a new WinUI Window.
Next change the baseclass from "Window" to SplashScreen:

Before:
```xml
<Window ...>
</Window>
```
After:
```xml
<winuiex:SplashScreen
    xmlns:winuiex="using:WinUIEx" ...>

</winuiex:SplashScreen>
```
And codebehind:

Before:
```cs
public sealed partial class SplashScreen : Window
{
        public SplashScreen()
        {
            this.InitializeComponent();
        }
}
```
After:
```cs
public sealed partial class SplashScreen : WinUIEx.SplashScreen
{
        public SplashScreen(Type window) : base(window)
        {
            this.InitializeComponent();
        }
}
```

Next in App.xaml.cs, change OnLaunched from:
```cs
        protected override void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
        {
            m_window = new MainWindow();
            m_window.Activate();
        }
```
To:
```cs
        protected override void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
        {
             var splash = new SplashScreen(typeof(MainWindow));
             splash.Completed += (s, e) => m_window = e;
        }
```

Lastly, override OnLoading, to create some long-running setup work - once this method completes, the splash screen will close and the window in the type parameter will launch.

Example:
```
       protected override async Task OnLoading()
       {
            //TODO: Do some actual work
            for (int i = 0; i < 100; i+=5)
            {
                statusText.Text = $"Loading {i}%...";
                progressBar.Value = i;
                await Task.Delay(50);
            }
       }
```