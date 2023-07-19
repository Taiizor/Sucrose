using CefSharp;
using System.Windows;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SESHS = Sucrose.Engine.Shared.Helper.Source;

namespace Sucrose.Engine.CS.Event
{
    internal static class Web
    {
        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            SECSMI.CefEngine.Address = SESHS.GetSource(SECSMI.Web).ToString();

            SECSMI.CefEngine.ShowDevTools();
        }
    }
}