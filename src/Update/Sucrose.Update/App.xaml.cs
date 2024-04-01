using Downloader;
using System.ComponentModel;
using System.Globalization;
using System.IO;
using System.Windows;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SEVT = Skylark.Enum.VersionType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SHN = Skylark.Helper.Numeric;
using SHV = Skylark.Helper.Versionly;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SRER = Sucrose.Resources.Extension.Resources;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSCECT = Sucrose.Shared.Core.Enum.ChannelType;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHU = Sucrose.Shared.Core.Helper.Update;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSDEUT = Sucrose.Shared.Dependency.Enum.UpdateType;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SSHG = Skylark.Standard.Helper.GitHub;
using SSIIA = Skylark.Standard.Interface.IAssets;
using SSIIR = Skylark.Standard.Interface.IReleases;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSSZEZ = Sucrose.Shared.SevenZip.Extension.Zip;
using SSSZHZ = Sucrose.Shared.SevenZip.Helper.Zip;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SUMI = Sucrose.Update.Manage.Internal;
using SUMM = Sucrose.Update.Manage.Manager;
using SUVDIB = Sucrose.Update.View.DarkInfoBox;
using SUVDUB = Sucrose.Update.View.DarkUpdateBox;
using SUVLIB = Sucrose.Update.View.LightInfoBox;
using SUVLUB = Sucrose.Update.View.LightUpdateBox;

namespace Sucrose.Update
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Bundle { get; set; } = string.Empty;

        private static bool HasBundle { get; set; } = false;

        private static bool HasError { get; set; } = true;

        private static bool HasInfo { get; set; } = false;

        private static bool HasFile { get; set; } = false;

        private static int MinDelay => 1000;

        public App()
        {
            AppDomain.CurrentDomain.FirstChanceException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_FirstChanceException(Exception);

                //Close();
                //Message(Exception.Message);
            };

            AppDomain.CurrentDomain.UnhandledException += async (s, e) =>
            {
                Exception Exception = (Exception)e.ExceptionObject;

                await SSWW.Watch_GlobalUnhandledException(Exception);

                //Close();
                Message(Exception.Message);
            };

            TaskScheduler.UnobservedTaskException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_UnobservedTaskException(Exception);

                e.SetObserved();

                //Close();
                Message(Exception.Message);
            };

            Current.DispatcherUnhandledException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_DispatcherUnhandledException(Exception);

                e.Handled = true;

                //Close();
                Message(Exception.Message);
            };

            SHC.All = new CultureInfo(SMMM.Culture, true);
        }

        protected void Close()
        {
            SMMI.UpdateSettingManager.SetSetting(SMC.UpdateState, false);

            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        internal void Info(SSDEUT Type)
        {
            if (!HasInfo)
            {
                if (Type != SSDEUT.Updating)
                {
                    HasInfo = true;
                }

                switch (SUMM.ThemeType)
                {
                    case SEWTT.Dark:
                        SUVDIB DarkInfoBox = new(Type);
                        DarkInfoBox.ShowDialog();
                        break;
                    default:
                        SUVLIB LightInfoBox = new(Type);
                        LightInfoBox.ShowDialog();
                        break;
                }
            }
        }

        protected static void UpdateLimit()
        {
            if (SMMM.UpdateLimitValue > 0)
            {
                double UpdateLimit = SSESSE.Convert(SMMM.UpdateLimitValue, SMMM.UpdateLimitType, SEST.Byte, SEMST.Palila);

                long Limit = Convert.ToInt64(SHN.Numeral(UpdateLimit, false, false, 0, '0', SECNT.None));

                SUMI.DownloadConfiguration.MaximumBytesPerSecond = Limit;
            }
            else
            {
                SUMI.DownloadConfiguration.MaximumBytesPerSecond = 0;
            }
        }

        protected void Message(string Message)
        {
            if (HasError)
            {
                HasError = false;

                string Source = SUMI.Source;
                string Path = SMMI.UpdateLogManager.LogFile();
                string Text = SRER.GetValue("Update", "HelpText");

                SSSHW.Start(Message, Path, Source, Text);

                Close();
            }
        }

        protected async void Configure()
        {
            if (Directory.Exists(SUMM.CachePath))
            {
                string[] Files = Directory.GetFiles(SUMM.CachePath);

                foreach (string Record in Files)
                {
                    File.Delete(Record);
                }
            }
            else
            {
                Directory.CreateDirectory(SUMM.CachePath);
            }

            await Task.Delay(MinDelay);

            if (SSSHN.GetHostEntry())
            {
                SSSHS.Apply();

                SMMI.UpdateSettingManager.SetSetting(SMC.UpdateTime, DateTime.Now);

                List<SSIIR> Releases = SSHG.ReleasesList(SMR.Owner, SMR.Repository, SMMM.UserAgent, SMMM.Key);

                if (Releases.Any())
                {
                    SSIIR Release = Releases.FirstOrDefault();

                    if (SUMM.ChannelType == SSCECT.Release)
                    {
                        Release = Releases.FirstOrDefault(Releasing => !Releasing.PreRelease);
                    }

                    if (Release == null)
                    {
                        Info(SSDEUT.Channel);
                    }
                    else
                    {
                        Version Current = new(SSCHV.GetText());
                        Version Latest = SHV.Clear(Release.TagName);

                        if (SHV.Compare(Current, Latest) == SEVT.Latest)
                        {
                            List<SSIIA> Assets = Release.Assets;

                            if (Assets.Any())
                            {
                                foreach (SSIIA Asset in Assets)
                                {
                                    string Name = $"{SMR.AppName}_{SMR.Bundle}_{SSCHF.GetDescription()}_{SSCHA.Get()}_{Latest}{SSCHU.GetDescription(SUMI.UpdateType)}";

                                    string[] Required =
                                    {
                                        SSCHU.GetDescription(SUMI.UpdateType),
                                        SSCHF.GetDescription(),
                                        SSCHA.GetText(),
                                        $"{Latest}",
                                        SMR.AppName,
                                        SMR.Bundle
                                    };

                                    if (Asset.Name.Contains(Name) && Required.All(Asset.Name.Contains))
                                    {
                                        Info(SSDEUT.Updating);

                                        SUMI.Source = Asset.BrowserDownloadUrl;

                                        Bundle = Path.Combine(SUMM.CachePath, Path.GetFileName(SUMI.Source));

                                        if (File.Exists(Bundle))
                                        {
                                            File.Delete(Bundle);
                                        }

                                        UpdateLimit();

                                        SUMI.DownloadService = new(SUMI.DownloadConfiguration);

                                        SUMI.DownloadService.DownloadStarted += OnDownloadStarted;
                                        SUMI.DownloadService.DownloadFileCompleted += OnDownloadFileCompleted;
                                        SUMI.DownloadService.DownloadProgressChanged += OnDownloadProgressChanged;
                                        SUMI.DownloadService.ChunkDownloadProgressChanged += OnChunkDownloadProgressChanged;

                                        await SUMI.DownloadService.DownloadFileTaskAsync(SUMI.Source, Bundle);

                                        break;
                                    }
                                }

                                if (string.IsNullOrEmpty(Bundle))
                                {
                                    Info(SSDEUT.Condition);
                                }
                            }
                            else
                            {
                                Info(SSDEUT.Empty);
                            }
                        }
                        else
                        {
                            Info(SSDEUT.Update);
                        }
                    }
                }
                else
                {
                    Info(SSDEUT.Releases);
                }
            }
            else
            {
                Info(SSDEUT.Network);
            }

            if (HasBundle)
            {
                if (SUMI.UpdateType == SSCEUT.Executable || SSSZHZ.CheckArchive(Bundle))
                {
                    SSDECT Result = SSDECT.Pass;

                    if (SUMI.UpdateType == SSCEUT.Compressed)
                    {
                        Result = SSSZEZ.Extract(Bundle, SUMM.CachePath);

                        if (Result == SSDECT.Pass)
                        {
                            await Task.Delay(MinDelay);

                            Bundle = Path.ChangeExtension(Bundle, SSCHU.GetDescription(SSCEUT.Executable));
                        }
                        else
                        {
                            Info(SSDEUT.Extract);
                        }
                    }

                    if (Result == SSDECT.Pass)
                    {
                        switch (SUMM.ThemeType)
                        {
                            case SEWTT.Dark:
                                SUVDUB DarkUpdateBox = new(Bundle);
                                DarkUpdateBox.ShowDialog();
                                break;
                            default:
                                SUVLUB LightUpdateBox = new(Bundle);
                                LightUpdateBox.ShowDialog();
                                break;
                        }
                    }
                }
                else
                {
                    Info(SSDEUT.Download);
                }
            }
            else
            {
                if (HasFile)
                {
                    Info(SSDEUT.Error);
                }
                else
                {
                    Info(SSDEUT.Cancelled);
                }
            }

            Close();
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            //

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (SSSHI.Basic(SMR.UpdateMutex, SMR.Update))
            {
                SMMI.UpdateSettingManager.SetSetting(SMC.UpdatePercentage, "0%");
                SMMI.UpdateSettingManager.SetSetting(SMC.UpdateState, false);

                Configure();
            }
            else
            {
                Close();
            }
        }

        private void OnDownloadStarted(object sender, DownloadStartedEventArgs e)
        {
            SMMI.UpdateSettingManager.SetSetting(SMC.UpdateState, true);
            HasBundle = true;
        }

        private void OnDownloadFileCompleted(object sender, AsyncCompletedEventArgs e)
        {
            if (e.Error != null)
            {
                HasBundle = false;
                HasFile = true;
            }
            else if (e.Cancelled)
            {
                HasBundle = false;
                HasFile = false;
            }
            else
            {
                SMMI.UpdateSettingManager.SetSetting(SMC.UpdatePercentage, $"{SHN.Numeral(100, true, true, 2, '0', SECNT.None)}%");
            }
        }

        private static void OnDownloadProgressChanged(object sender, DownloadProgressChangedEventArgs e)
        {
            string Percentage = $"{SHN.Numeral(e.ProgressPercentage, true, true, 2, '0', SECNT.None)}%";

            if (SMMM.UpdatePercentage != Percentage)
            {
                SMMI.UpdateSettingManager.SetSetting(SMC.UpdatePercentage, Percentage);
            }

            UpdateLimit();
        }

        private static void OnChunkDownloadProgressChanged(object sender, DownloadProgressChangedEventArgs e)
        {
            UpdateLimit();
        }
    }
}