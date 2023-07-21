using System.Windows;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;

namespace Sucrose.Engine.CS.Event
{
    internal static class Web
    {
        public static void CefEngineInitializedChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            SESMI.Initialized = SECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            SECSMI.CefEngine.Address = SESHS.GetSource(SECSMI.Web).ToString();

            //SECSMI.CefEngine.ShowDevTools();
        }
    }
}