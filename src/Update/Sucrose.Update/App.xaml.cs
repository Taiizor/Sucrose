using System.Globalization;
using System.IO;
using System.Net.Http;
using System.Windows;
using SEVT = Skylark.Enum.VersionType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SHV = Skylark.Helper.Versionly;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHU = Sucrose.Shared.Core.Helper.Update;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;
using SSDEUT = Sucrose.Shared.Dependency.Enum.UpdateType;
using SSHG = Skylark.Standard.Helper.GitHub;
using SSIIA = Skylark.Standard.Interface.IAssets;
using SSIIR = Skylark.Standard.Interface.IReleases;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSRHR = Sucrose.Shared.Resources.Helper.Resources;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSSZEZ = Sucrose.Shared.SevenZip.Extension.Zip;
using SSSZHZ = Sucrose.Shared.SevenZip.Helper.Zip;
using SSWDEMB = Sucrose.Shared.Watchdog.DarkErrorMessageBox;
using SSWLEMB = Sucrose.Shared.Watchdog.LightErrorMessageBox;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SUMI = Sucrose.Update.Manage.Internal;
using SUMM = Sucrose.Update.Manage.Manager;
using SUVDIB = Sucrose.Update.View.DarkInfoBox;
using SUVDUB = Sucrose.Update.View.DarkUpdateBox;
using SUVLIB = Sucrose.Update.View.LightInfoBox;
using SUVLUB = Sucrose.Update.View.LightUpdateBox;
using SWUSI = Skylark.Wing.Utility.SingleInstance;

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

                SSWW.Watch_GlobalUnhandledException(Exception);

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

            SHC.All = new CultureInfo(SMMM.Culture, true);
        }

        protected void Close()
        {
            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        internal void Info(SSDEUT Type)
        {
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

        protected void Message(string Message)
        {
            if (HasError)
            {
                HasError = false;

                string Source = SUMI.Source;
                string Path = SMMI.UpdateLogManager.LogFile();
                string Text = SSRER.GetValue("Update", "HelpText");

                switch (SUMM.ThemeType)
                {
                    case SEWTT.Dark:
                        SSWDEMB DarkMessageBox = new(Message, Path, Source, Text)
                        {
                            Topmost = true
                        };
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        SSWLEMB LightMessageBox = new(Message, Path, Source, Text)
                        {
                            Topmost = true
                        };
                        LightMessageBox.ShowDialog();
                        break;
                }

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

                List<SSIIR> Releases = SSHG.ReleasesList(SMR.Owner, SMR.Repository, SMMM.UserAgent, SMMM.Key);

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

                                    using HttpClient Client = new()
                                    {
                                        Timeout = Timeout.InfiniteTimeSpan
                                    };

                                    Client.DefaultRequestHeaders.Add("User-Agent", SMMM.UserAgent);

                                    using HttpResponseMessage Response = await Client.GetAsync(SUMI.Source);

                                    Response.EnsureSuccessStatusCode();

                                    await Task.Delay(MinDelay);

                                    if (Response.IsSuccessStatusCode)
                                    {
                                        using HttpContent Content = Response.Content;

                                        using Stream CStream = await Content.ReadAsStreamAsync();
                                        using FileStream FStream = new(Bundle, FileMode.OpenOrCreate, FileAccess.Write, FileShare.None);

                                        await Task.Delay(MinDelay);

                                        await CStream.CopyToAsync(FStream);

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

            SSRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (!SWUSI.IsAppMutexRunning(SMR.UpdateMutex) && SSSHP.WorkCount(SMR.Update) <= 1)
            {
                SUMI.Mutex.ReleaseMutex();

                Configure();
            }
            else
            {
                Close();
            }
        }
    }
}