#if LAUNCHER

using System.IO;
using SSIL = Sucrose.Signal.Interface.Launcher;
using SSLMI = Sucrose.Shared.Launcher.Manage.Internal;
using SSMI = Sucrose.Signal.Manage.Internal;

namespace Sucrose.Shared.Signal.Services
{
    public static class LauncherSignalService
    {
        public static async void Handler(object sender, FileSystemEventArgs e)
        {
            SSIL Data = await SSMI.LauncherManager.FileRead<SSIL>(e.FullPath, new());

            if (Data.Hide)
            {
                SSLMI.TrayIconManager.Hide();
            }

            if (Data.Show)
            {
                SSLMI.TrayIconManager.Show();
            }

            if (Data.Release)
            {
                SSLMI.TrayIconManager.Release();
            }
        }
    }
}

#endif