using System.Globalization;
using System.IO;
using System.Net.Http;
using System.Windows;
using SEVT = Skylark.Enum.VersionType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SHV = Skylark.Helper.Versionly;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSDEUT = Sucrose.Shared.Dependency.Enum.UpdateType;
using SSHG = Skylark.Standard.Helper.GitHub;
using SSIIA = Skylark.Standard.Interface.IAssets;
using SSIIR = Skylark.Standard.Interface.IReleases;
using SSRHR = Sucrose.Shared.Resources.Helper.Resources;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSWDEMB = Sucrose.Shared.Watchdog.DarkErrorMessageBox;
using SSWLEMB = Sucrose.Shared.Watchdog.LightErrorMessageBox;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SUVDIB = Sucrose.Update.View.DarkInfoBox;
using SUVDUB = Sucrose.Update.View.DarkUpdateBox;
using SUVLIB = Sucrose.Update.View.LightInfoBox;
using SUVLUB = Sucrose.Update.View.LightUpdateBox;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Update
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SHC.CurrentUITwoLetterISOLanguageName);

        private static string CachePath => Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Bundle);

        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static string Agent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        private static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        private static string Bundle { get; set; } = string.Empty;

        private static Mutex Mutex => new(true, SMR.UpdateMutex);

        private static bool HasBundle { get; set; } = false;

        private static bool HasError { get; set; } = true;

        private static int MaxDelay => 3000;

        private static int MinDelay => 1000;

        public App()
        {
            AppDomain.CurrentDomain.FirstChanceException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SSWW.Watch_FirstChanceException(Exception);

                //Close();
                //Message(Exception.Message);
            };

            AppDomain.CurrentDomain.UnhandledException += (s, e) =>
            {
                Exception Exception = (Exception)e.ExceptionObject;

                SSWW.Watch_GlobalUnhandledExceptionHandler(Exception);

                //Close();
                Message(Exception.Message);
            };

            TaskScheduler.UnobservedTaskException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SSWW.Watch_UnobservedTaskException(Exception);

                e.SetObserved();

                //Close();
                Message(Exception.Message);
            };

            Current.DispatcherUnhandledException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SSWW.Watch_DispatcherUnhandledException(Exception);

                e.Handled = true;

                //Close();
                Message(Exception.Message);
            };

            SHC.All = new CultureInfo(Culture, true);
        }

        protected void Close()
        {
            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        internal void Info(SSDEUT Type)
        {
            switch (Theme)
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

        protected void Message(string Message)
        {
            if (HasError)
            {
                HasError = false;

                string Path = SMMI.UpdateLogManager.LogFile();

                switch (Theme)
                {
                    case SEWTT.Dark:
                        SSWDEMB DarkMessageBox = new(Message, Path);
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        SSWLEMB LightMessageBox = new(Message, Path);
                        LightMessageBox.ShowDialog();
                        break;
                }

                Close();
            }
        }

        protected async void Configure()
        {
            if (Directory.Exists(CachePath))
            {
                string[] Files = Directory.GetFiles(CachePath);

                foreach (string Record in Files)
                {
                    File.Delete(Record);
                }
            }
            else
            {
                Directory.CreateDirectory(CachePath);
            }

            await Task.Delay(MinDelay);

            if (SSSHN.GetHostEntry())
            {
                SSSHS.Apply();

                List<SSIIR> Releases = SSHG.ReleasesList(SMR.Owner, SMR.Repository, Agent, Key);

                if (Releases.Any())
                {
                    SSIIR Release = Releases.First();

                    Version Current = new(SSCHV.GetText());
                    Version Latest = SHV.Clear(Release.TagName);

                    if (SHV.Compare(Current, Latest) == SEVT.Latest)
                    {
                        List<SSIIA> Assets = Release.Assets;

                        if (Assets.Any())
                        {
                            foreach (SSIIA Asset in Assets)
                            {
                                string Name = $"{SMR.AppName}_{SMR.Bundle}_{SSCHF.GetDescription()}_{SSCHA.Get()}_{Latest}";

                                string[] Required =
                                {
                                    SSCHF.GetDescription(),
                                    SSCHA.GetText(),
                                    $"{Latest}",
                                    SMR.AppName,
                                    SMR.Bundle
                                };

                                if (Asset.Name.Contains(Name) && Required.All(Asset.Name.Contains))
                                {
                                    Info(SSDEUT.Updating);

                                    string Source = Asset.BrowserDownloadUrl;

                                    Bundle = Path.Combine(CachePath, Path.GetFileName(Source));

                                    if (File.Exists(Bundle))
                                    {
                                        File.Delete(Bundle);
                                    }

                                    using HttpClient Client = new()
                                    {
                                        Timeout = Timeout.InfiniteTimeSpan
                                    };

                                    Client.DefaultRequestHeaders.Add("User-Agent", Agent);

                                    using HttpResponseMessage Response = await Client.GetAsync(Source);

                                    Response.EnsureSuccessStatusCode();

                                    await Task.Delay(MinDelay);

                                    if (Response.IsSuccessStatusCode)
                                    {
                                        using HttpContent Content = Response.Content;

                                        using Stream CStream = await Content.ReadAsStreamAsync();
                                        using FileStream FStream = new(Bundle, FileMode.OpenOrCreate, FileAccess.Write, FileShare.None);

                                        await Task.Delay(MaxDelay);

                                        HasBundle = true;

                                        break;
                                    }
                                    else
                                    {
                                        Info(SSDEUT.Status);
                                    }
                                }
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
                switch (Theme)
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

            SSRHR.SetLanguage(Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (Mutex.WaitOne(TimeSpan.Zero, true) && SSSHP.WorkCount(SMR.Update) <= 1)
            {
                Mutex.ReleaseMutex();

                Configure();
            }
            else
            {
                Close();
            }
        }
    }
}