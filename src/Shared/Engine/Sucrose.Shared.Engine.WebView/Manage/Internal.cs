using WebEngine = Microsoft.Web.WebView2.Wpf.WebView2;

namespace Sucrose.Shared.Engine.WebView.Manage
{
    internal static class Internal
    {
        public static int Try = 0;

        public static bool State = true;

        public static string Gif = string.Empty;

        public static string Url = string.Empty;

        public static string Web = string.Empty;

        public static string Video = string.Empty;

        public static WebEngine WebEngine = new();

        public static List<int> Processes = new();

        public static string YouTube = string.Empty;

        public static IntPtr WebHandle = IntPtr.Zero;
    }
}