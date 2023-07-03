using Skylark.Enum;
using Skylark.Wing;
using Sucrose.Player.CefSharp;
using Sucrose.Player.WebView2;
using Sucrose.Player.MediaElement;
using System.Windows;

namespace Sucrose.Test
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        public App()
        {
            var Player = new CefSharp();
            Player.Show();

            Engine.WallpaperWindow(Player, 0, ScreenType.DisplayBound);
        }
    }
}