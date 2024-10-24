﻿#if LAUNCHER

using System.IO;
using SPIL = Sucrose.Pipe.Interface.Launcher;
using SSLMI = Sucrose.Shared.Launcher.Manage.Internal;
using SPMI = Sucrose.Pipe.Manage.Internal;

namespace Sucrose.Shared.Pipe.Services
{
    public static class LauncherPipeService
    {
        public static void Handler(SPEMREA e)
        {
            try
            {
                if (e != null && !string.IsNullOrEmpty(e.Message))
                {
                    SPIL Data = JsonConvert.DeserializeObject<SPIL>(e.Message);

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
            catch { }
        }
    }
}

#endif