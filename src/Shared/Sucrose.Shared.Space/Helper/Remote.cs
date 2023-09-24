namespace Sucrose.Shared.Space.Helper
{
    internal static class Remote
    {
        public static List<string> GetApp()
        {
            return new()
            {
                "rdp.exe",
                "rdpv.exe",
                "mstsc.exe",
                "radmin.exe",
                "anydesk.exe",
                "alpemix.exe",
                "rdpclip.exe",
                "rdpinit.exe",
                "tightvnc.exe",
                "rustdesk.exe",
                "rdpshell.exe",
                "rdpviewer.exe",
                "teamviewer.exe",
                "quickassist.exe",
                "ultravnc_viewer.exe"
            };
        }
    }
}