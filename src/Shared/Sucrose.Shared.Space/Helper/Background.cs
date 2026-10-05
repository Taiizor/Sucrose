using Sucrose.Shared.Theme.Helper;
using System.IO;
using SHV = Skylark.Helper.Versionly;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMML = Sucrose.Manager.Manage.Library;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;
using SWHBI = Skylark.Wing.Helper.BackgroundImage;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Background
    {
        public static async void SetSetting()
        {
            try
            {
                string BackgroundImagePath = SWHBI.GetPathSystemParameters();

                if (!BackgroundImagePath.StartsWith(SMML.Location))
                {
                    SMMI.EngineSettingManager.SetSetting(SMMCE.BackgroundImagePath, BackgroundImagePath);
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void ResetDefault()
        {
            try
            {
                string BackgroundImagePath = SMME.BackgroundImagePath;
                string DefaultImagePath = SWHBI.GetPathSystemParameters();

                if (!string.IsNullOrWhiteSpace(SMME.BackgroundImagePath) && File.Exists(BackgroundImagePath) && DefaultImagePath != BackgroundImagePath)
                {
                    SWHBI.SetPathSystemParameters(BackgroundImagePath, false);

                    SWHBI.SetStyleRegistry(Refresh: false);
                    SWHBI.SetTileRegistry(Refresh: false);

                    SWUD.RefreshDesktop();
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void SetWallpaper(string Wallpaper = null)
        {
            try
            {
                if (string.IsNullOrWhiteSpace(Wallpaper))
                {
                    string InfoPath = Path.Combine(SMML.Location, SMML.Selected, SMMRC.SucroseInfo);

                    if (File.Exists(InfoPath) && SSTHI.ReadCheck(InfoPath))
                    {
                        SSTHI Info = SSTHI.ReadJson(InfoPath);

                        if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                        {
                            Wallpaper = Path.Combine(SMML.Location, SMML.Selected, Info.Thumbnail);
                        }
                    }
                }

                if (!string.IsNullOrWhiteSpace(Wallpaper) && File.Exists(Wallpaper))
                {
                    SetSetting();

                    SWHBI.SetPathSystemParameters(Wallpaper, false);

                    SWHBI.SetStyleRegistry(Refresh: false);
                    SWHBI.SetTileRegistry(Refresh: false);

                    if (SMME.DisableSlideshow)
                    {
                        SWHBI.DisableSlideshow();
                    }

                    SWUD.RefreshDesktop();
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }
    }
}