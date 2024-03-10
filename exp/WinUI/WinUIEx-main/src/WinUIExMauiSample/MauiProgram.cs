using Microsoft.Maui.LifecycleEvents;
using Microsoft.UI.Composition.SystemBackdrops;
using Microsoft.UI.Xaml.Media;
using WinUIEx;

namespace WinUIExMauiSample
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
                })
#if WINDOWS
                 .ConfigureLifecycleEvents(events =>
                 {
                     events.AddWindows(wndLifeCycleBuilder =>
                     {
                         wndLifeCycleBuilder.OnWindowCreated(window =>
                         {
                             var manager = WinUIEx.WindowManager.Get(window);
                             manager.PersistenceId = "MainWindowPersistanceId";
                             manager.MinWidth = 640;
                             manager.MinHeight = 480;
                             window.SystemBackdrop = new DesktopAcrylicBackdrop();
                         });
                     });
                 })
#endif
                 ;

            return builder.Build();
        }
    }
}