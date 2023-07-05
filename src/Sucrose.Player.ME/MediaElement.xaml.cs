using System.IO;
using System.Text.RegularExpressions;
using System.Windows;
using SPMEEH = Sucrose.Player.ME.Event.Handler;
using SPMEHMEH = Sucrose.Player.ME.Helper.MediaElementHelper;
using SPMEMI = Sucrose.Player.ME.Manage.Internal;
using SPSEH = Sucrose.Player.Shared.Event.Handler;
using SEST = Skylark.Enum.ScreenType;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SSEDT = Sucrose.Space.Enum.DisplayType;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SWE = Skylark.Wing.Engine;
using System.Windows.Threading;
#if NET48_OR_GREATER
using System.Net.Http;
using SMR = Sucrose.Memory.Readonly;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
#endif

namespace Sucrose.Player.ME
{
    /// <summary>
    /// Interaction logic for MediaElement.xaml
    /// </summary>
    public sealed partial class MediaElement : Window
    {
        private readonly DispatcherTimer Timer = new();

        public MediaElement()
        {
            InitializeComponent();

            Content = SPMEMI.MediaPlayer;

            SPMEMI.MediaPlayer.Source = GetSource(SMMI.EngineSettingManager.GetSetting(SMC.Video, @""));

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SPMEMI.MediaPlayer.MediaOpened += SPMEEH.MediaPlayerOpened;
            SPMEMI.MediaPlayer.MediaFailed += SPMEEH.MediaPlayerFailed;
            SPMEMI.MediaPlayer.MediaEnded += SPMEEH.MediaPlayerEnded;

            Closing += (s, e) => SPMEMI.MediaPlayer.Close();
            Loaded += (s, e) => SPSEH.WindowLoaded(this);

            SPMEHMEH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            SPMEHMEH.Play();
        }

        private static Uri GetSource(Uri Source)
        {
            return GetSource(Source.ToString());
        }

        private static Uri GetSource(string Source)
        {
            if (IsUrl(Source))
            {
#if NET48_OR_GREATER
                string CachePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.MediaElement);

                if (!Directory.Exists(CachePath))
                {
                    Directory.CreateDirectory(CachePath);
                }

                //string LocalSource = @Path.Combine(CachePath, Path.ChangeExtension(Path.GetRandomFileName(), Path.GetExtension(Source)));
                string LocalSource = @Path.Combine(CachePath, $"{SSECCE.TextToMD5(Source)}{Path.GetExtension(Source)}");

                if (File.Exists(LocalSource))
                {
                    return new Uri(@LocalSource, UriKind.RelativeOrAbsolute);
                }
                else
                {
                    using HttpClient Client = new();
                    using HttpResponseMessage Response = Client.GetAsync(Source).Result;
                    using Stream Content = Response.Content.ReadAsStreamAsync().Result;
                    using FileStream Stream = new(LocalSource, FileMode.OpenOrCreate, FileAccess.Write, FileShare.None);

                    Content.CopyTo(Stream);

                    return new Uri(@Path.GetFullPath(LocalSource), UriKind.RelativeOrAbsolute);
                }
#else
                return new Uri(@Source, UriKind.RelativeOrAbsolute);
#endif
            }
            else
            {
                return new Uri(@Source, UriKind.RelativeOrAbsolute);
            }
        }

        private static bool IsUrl(string Address)
        {
            string Pattern = @"^(http|https|ftp)://[\w-]+(\.[\w-]+)+([\w.,@?^=%&:/~+#-]*[\w@?^=%&/~+#-])?$";

            Regex Regex = new(Pattern, RegexOptions.IgnoreCase);

            return Regex.IsMatch(Address);
        }

        private void MediaElement_ContentRendered(object sender, EventArgs e)
        {
            switch (SMMI.EngineSettingManager.GetSetting(SMC.DisplayType, SSEDT.Screen))
            {
                case SSEDT.Expand:
                    SWE.WallpaperWindow(this, SMMI.EngineSettingManager.GetSetting(SMC.ExpandScreenType, SEEST.Default), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
                case SSEDT.Duplicate:
                    SWE.WallpaperWindow(this, SMMI.EngineSettingManager.GetSetting(SMC.DuplicateScreenType, SEDST.Default), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
                default:
                    SWE.WallpaperWindow(this, SMMI.EngineSettingManager.GetSettingStable(SMC.ScreenIndex, 0), SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound));
                    break;
            }
        }

        private void Timer_Tick(object sender, EventArgs e)
        {
            SPMEHMEH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));

            Uri Video = GetSource(SMMI.EngineSettingManager.GetSetting(SMC.Video, new Uri(@"", UriKind.RelativeOrAbsolute)));

            if (SPMEMI.MediaPlayer.Source != Video)
            {
                SPMEMI.MediaPlayer.Source = Video;
            }
        }
    }
}