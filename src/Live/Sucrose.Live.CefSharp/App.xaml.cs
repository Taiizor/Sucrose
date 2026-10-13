using CefSharp;
using CefSharp.Wpf.HwndHost;
using Microsoft.Win32;
using System.Collections;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Net.Http;
using System.Windows;
using Application = System.Windows.Application;
using SEVT = Skylark.Enum.VersionType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SHV = Skylark.Helper.Versionly;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMCW = Sucrose.Memory.Manage.Constant.Warehouse;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMG = Sucrose.Manager.Manage.General;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMML = Sucrose.Manager.Manage.Library;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRM = Sucrose.Memory.Manage.Readonly.Mutex;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;
using SMMVL = Sucrose.Memory.Manage.Valuable.Log;
using SMMW = Sucrose.Manager.Manage.Warehouse;
using SRER = Sucrose.Resources.Extension.Resources;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DialogType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PropertiesType;
using SSDEPROXYT = Sucrose.Shared.Dependency.Enum.ProxyType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSDHG = Sucrose.Shared.Dependency.Helper.Graphic;
using SSDHR = Sucrose.Shared.Dependency.Helper.Runtime;
using SSDMI = Sucrose.Shared.Dependency.Manage.Internal;
using SSDMMG = Sucrose.Shared.Dependency.Manage.Manager.General;
using SSECSEWWS = Sucrose.Shared.Engine.CefSharp.Extension.WatsonWebServer;
using SSECSHP = Sucrose.Shared.Engine.CefSharp.Helper.Properties;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSECSVG = Sucrose.Shared.Engine.CefSharp.View.Gif;
using SSECSVU = Sucrose.Shared.Engine.CefSharp.View.Url;
using SSECSVV = Sucrose.Shared.Engine.CefSharp.View.Video;
using SSECSVW = Sucrose.Shared.Engine.CefSharp.View.Web;
using SSECSVYT = Sucrose.Shared.Engine.CefSharp.View.YouTube;
using SSEHA = Sucrose.Shared.Engine.Helper.Awakening;
using SSEHCR = Sucrose.Shared.Engine.Helper.Crashing;
using SSEHCY = Sucrose.Shared.Engine.Helper.Cycyling;
using SSEHP = Sucrose.Shared.Engine.Helper.Properties;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEVDMB = Sucrose.Shared.Engine.View.DarkMessageBox;
using SSEVLMB = Sucrose.Shared.Engine.View.LightMessageBox;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSHC = Sucrose.Shared.Space.Helper.Cycyling;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSSHWG = Sucrose.Shared.Space.Helper.Watchdog;
using SSSHWS = Sucrose.Shared.Space.Helper.Windows;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTHC = Sucrose.Shared.Theme.Helper.Compatible;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;
using SSWHD = Sucrose.Shared.Watchdog.Helper.Dataset;
using SWHA = System.Windows.HorizontalAlignment;
using SWVA = System.Windows.VerticalAlignment;

namespace Sucrose.Live.CefSharp
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static bool HasError { get; set; } = true;
        private static SSECSEWWS LocalServer { get; set; }

        public App()
        {
            System.Windows.Forms.Application.SetUnhandledExceptionMode(UnhandledExceptionMode.Automatic);

            System.Windows.Forms.Application.ThreadException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWEW.Watch_ThreadException(Exception);

                SSWHD.Add("Exception Type", "Thread Exception");

                Message(Exception, true);
                //Close();
            };

            AppDomain.CurrentDomain.FirstChanceException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWEW.Watch_FirstChanceException(Exception);

                SSWHD.Add("Exception Type", "First Chance Exception");

                //Message(Exception, false);
                //Close();
            };

            AppDomain.CurrentDomain.UnhandledException += async (s, e) =>
            {
                Exception Exception = (Exception)e.ExceptionObject;

                await SSWEW.Watch_GlobalUnhandledException(Exception);

                SSWHD.Add("Exception Type", "Global Unhand Exception");

                Message(Exception, true);
                //Close();
            };

            TaskScheduler.UnobservedTaskException += async (s, e) =>
            {
                e.SetObserved();

                Exception Exception = e.Exception;

                await SSWEW.Watch_UnobservedTaskException(Exception);

                SSWHD.Add("Exception Type", "Unobserved Task Exception");

                Message(Exception, false);
                //Close();
            };

            Current.DispatcherUnhandledException += async (s, e) =>
            {
                e.Handled = true;

                Exception Exception = e.Exception;

                await SSWEW.Watch_DispatcherUnhandledException(Exception);

                SSWHD.Add("Exception Type", "Dispatcher Unhandled Exception");

                Message(Exception, true);
                //Close();
            };

            SHC.All = new CultureInfo(SMMG.Culture, true);

            SSDHG.Configure(SSSMI.CefSharpProcess);

            SSDHR.Configure();

            SSDHG.Configure();
        }

        protected void Close()
        {
            LocalServer?.Stop();
            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Message(Exception Exception, bool Show)
        {
            if (HasError)
            {
                HasError = !Show;

                string Path = SMMI.CefSharpLiveLogManager.LogFile();

                SSSHWG.Start(SMMRA.CefSharpLive, Exception, Show, Path);

                if (Show)
                {
                    Close();
                }
            }
        }

        protected bool Check()
        {
            try
            {
#if X86
                string KeyPath = @"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x86";
#elif X64
                string KeyPath = @"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64";
#else
                string KeyPath = @"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\ARM64";
#endif

                using RegistryKey Key = Registry.LocalMachine.OpenSubKey(KeyPath, false);

                if (Key != null)
                {
                    object Version = Key.GetValue("Version");
                    object Installed = Key.GetValue("Installed");

                    SSWHD.Add("VC Redist Version", $"{Version}");
                    SSWHD.Add("VC Redist Installed", $"{Installed}");

                    if (Version != null && Installed != null && (int)Installed == 1)
                    {
                        if (SHV.Compare(SHV.Clear($"{Version}"), SHV.Clear("v14.40.33816.0")) != SEVT.Latest)
                        {
                            return true;
                        }
                        else
                        {
                            return false;
                        }
                    }
                    else
                    {
                        return false;
                    }
                }
                else
                {
                    return false;
                }
            }
            catch (Exception Exception)
            {
                SSWHD.Add("VC Redist Check Exception", new Hashtable()
                {
                    { "Message", Exception.Message },
                    { "Inner Exception", Exception.InnerException?.Message }
                });

                return false;
            }
        }

        protected void Checker()
        {
            SSWHD.Add("CefSharp Checker", new Hashtable()
            {
                { "Check", Check() },
                { "CefSharp Time", SMMW.CefSharpTime },
                { "CefSharp Time Check", SMMW.CefSharpTime > DateTime.Now }
            });

            if (Check() || SMMW.CefSharpTime > DateTime.Now)
            {
                Configure();
            }
            else
            {
                SSDEDT DialogResult;

                bool Check = SMMW.CefsharpContinue;

                string CloseText = SRER.GetValue("Live", "Close");
                string ContinueText = SRER.GetValue("Live", "Continue");
                string DownloadText = SRER.GetValue("Live", "Download");
                string RememberText = SRER.GetValue("Live", "Remember");

                string DialogInfo = SRER.GetValue("Live", "Info", "CefSharp");
                string DialogMessage = SRER.GetValue("Live", "Message", "CefSharp");

                string DialogTitle = string.Format(SRER.GetValue("Live", "Title"), "CefSharp");

                switch (SSDMMG.ThemeType)
                {
                    case SEWTT.Dark:
                        SSEVDMB DarkMessageBox = new(DialogTitle, DialogMessage, DialogInfo, RememberText, DownloadText, ContinueText, CloseText, Check);
                        DarkMessageBox.ShowDialog();

                        DialogResult = DarkMessageBox.Result;
                        break;
                    default:
                        SSEVLMB LightMessageBox = new(DialogTitle, DialogMessage, DialogInfo, RememberText, DownloadText, ContinueText, CloseText, Check);
                        LightMessageBox.ShowDialog();

                        DialogResult = LightMessageBox.Result;
                        break;
                }

                SSWHD.Add("CefSharp Dialog Result", $"{DialogResult}");

                switch (DialogResult)
                {
                    case SSDEDT.Continue:
                        Configure();
                        break;
                    case SSDEDT.Download:
                        SMMI.WarehouseSettingManager.SetSetting(SMMCW.CefsharpContinue, true);
                        Downloader();
                        break;
                    case SSDEDT.Remember:
                        Remember();
                        break;
                    default:
                        Close();
                        break;
                }
            }
        }

        protected void Remember()
        {
            SMMI.WarehouseSettingManager.SetSetting(SMMCW.CefSharpTime, DateTime.Now.AddDays(1));

            Configure();
        }

        protected async void Configure()
        {
            SSEMI.LibraryLocation = SMML.Location;
            SSEMI.LibrarySelected = SMML.Selected;

            if (SMMI.LibrarySettingManager.CheckFile() && !string.IsNullOrEmpty(SSEMI.LibrarySelected))
            {
                SSEMI.InfoPath = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, SMMRC.SucroseInfo);
                SSECSMI.CefPath = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.CefSharp);
                SSEMI.CompatiblePath = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, SMMRC.SucroseCompatible);
                SSEMI.PropertiesPath = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, SMMRC.SucroseProperties);

                if (File.Exists(SSEMI.InfoPath) && SSTHI.ReadCheck(SSEMI.InfoPath))
                {
                    SSEMI.Info = SSTHI.ReadJson(SSEMI.InfoPath);

                    if (SSEMI.Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                    {
                        SSLHK.StopSubprocess();

                        CefSettings Settings = new()
                        {
                            UserAgent = SMMG.UserAgent,
                            CachePath = SSECSMI.CefPath,
                            PersistSessionCookies = true,
                            IgnoreCertificateErrors = true,
                            LocalesDirPath = SSSMI.Locales,
                            LogSeverity = LogSeverity.Default,
                            WindowlessRenderingEnabled = true,
                            LogFile = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Log, string.Format("CefSharpNative-{0}-{1}.log", SMMVL.FileNameDate, Guid.NewGuid()))
                        };

                        if (SMME.ProxyEnabled && !string.IsNullOrEmpty(SMME.ProxyServer) && SMME.ProxyPort > 0)
                        {
                            string proxyScheme = SMME.ProxyType switch
                            {
                                SSDEPROXYT.HTTP => "http",
                                SSDEPROXYT.HTTPS => "https",
                                SSDEPROXYT.SOCKS5 => "socks5",
                                _ => "http"
                            };

                            string proxyUrl = $"{proxyScheme}://{SMME.ProxyServer}:{SMME.ProxyPort}";
                            
                            Settings.CefCommandLineArgs.Add("proxy-server", proxyUrl);

                            if (!string.IsNullOrEmpty(SMME.ProxyUsername))
                            {
                                // CefSharp doesn't support proxy authentication directly via command line
                                // Authentication would need to be handled via IRequestHandler
                            }
                        }

                        SSEMI.BrowserSettings.CefSharp = SMME.CefArguments;

                        if (!SSEMI.BrowserSettings.CefSharp.Any())
                        {
                            SSEMI.BrowserSettings.CefSharp = SSEMI.CefArguments;

                            SMMI.EngineSettingManager.SetSetting(SMMCE.CefArguments, SSEMI.BrowserSettings.CefSharp);
                        }

                        foreach (KeyValuePair<string, string> Argument in SSEMI.BrowserSettings.CefSharp)
                        {
                            if (Settings.CefCommandLineArgs.ContainsKey(Argument.Key))
                            {
                                Settings.CefCommandLineArgs[Argument.Key] = Argument.Value;
                            }
                            else
                            {
                                Settings.CefCommandLineArgs.Add(Argument.Key, Argument.Value);
                            }
                        }

                        if (SSSHWS.IsGermanium())
                        {
                            if (Settings.CefCommandLineArgs.ContainsKey("disable-gpu-compositing"))
                            {
                                Settings.CefCommandLineArgs.Remove("disable-gpu-compositing");
                            }
                        }

                        if (SMME.DeveloperPort > 0)
                        {
                            Settings.RemoteDebuggingPort = SMME.DeveloperPort;

                            if (!Settings.CefCommandLineArgs.ContainsKey("remote-allow-origins"))
                            {
                                Settings.CefCommandLineArgs.Add("remote-allow-origins", "*");
                            }
                        }

                        if (SMME.HardwareAcceleration)
                        {
                            if (!Settings.CefCommandLineArgs.ContainsKey("enable-gpu"))
                            {
                                Settings.CefCommandLineArgs.Add("enable-gpu", "1");
                            }

                            if (!Settings.CefCommandLineArgs.ContainsKey("enable-gpu-vsync"))
                            {
                                Settings.CefCommandLineArgs.Add("enable-gpu-vsync", "1");
                            }
                        }
                        else if (!SSSHWS.IsGermanium())
                        {
                            if (!Settings.CefCommandLineArgs.ContainsKey("disable-gpu"))
                            {
                                Settings.CefCommandLineArgs.Add("disable-gpu", "1");
                            }

                            if (!Settings.CefCommandLineArgs.ContainsKey("disable-gpu-vsync"))
                            {
                                Settings.CefCommandLineArgs.Add("disable-gpu-vsync", "1");
                            }
                        }

                        if (Directory.Exists(SSSMI.Locales))
                        {
                            string Locales = Directory.GetFiles(SSSMI.Locales, "*.pak")
                                .FirstOrDefault(Locale => Path.GetFileNameWithoutExtension(Locale)
                                .StartsWith(SMMG.Culture, StringComparison.OrdinalIgnoreCase));

                            if (!string.IsNullOrEmpty(Locales))
                            {
                                Settings.Locale = Path.GetFileNameWithoutExtension(Locales);
                            }
                        }

                        //Settings.RegisterScheme(new CefCustomScheme
                        //{
                        //    SchemeName = "themeFolder",
                        //    IsDisplayIsolated = true,
                        //    IsFetchEnabled = true,
                        //    IsCSPBypassing = true,
                        //    IsCorsEnabled = true,
                        //    IsStandard = true,
                        //    IsSecure = false,
                        //    IsLocal = true,
                        //    SchemeHandlerFactory = new FolderSchemeHandlerFactory(
                        //        rootFolder: Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected),
                        //        defaultPage: SSEMI.Info.Source,
                        //        hostName: SSEMI.Info.Source)
                        //});

                        //Example of checking if a call to Cef.Initialize has already been made, we require this for
                        //our .Net 5.0 Single File Publish example, you don't typically need to perform this check
                        //if you call Cef.Initialze within your WPF App constructor.
                        if (Cef.IsInitialized is null or false)
                        {
                            //Perform dependency check to make sure all relevant resources are in our output directory.
                            await Cef.InitializeAsync(Settings, performDependencyCheck: true, browserProcessHandler: null);
                        }

                        string Source = SSEMI.Info.Source;

                        if (!SSTHV.IsUrl(Source))
                        {
                            Source = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, Source);
                        }

                        SMMI.BackgroundogSettingManager.SetSetting(new KeyValuePair<string, bool>[]
                        {
                            new(SMMCB.PipeRequired, false),
                            new(SMMCB.AudioRequired, false),
                            new(SMMCB.SignalRequired, false),
                            new(SMMCB.PausePerformance, false),
                            new(SMMCB.TransmissionRequired, false)
                        });

                        if (SSTHV.IsUrl(Source) || File.Exists(Source))
                        {
                            SSEHA.Start();

                            SSEHCR.Start();

                            SSEHCY.Start();

                            if (SSEMI.Info.Type is SSDEWT.Gif or SSDEWT.Video or SSDEWT.YouTube)
                            {
                                SSECSHP.Start();
                            }
                            else
                            {
                                SSEMI.PropertiesType = SSDEPT.Base;
                            }

                            if (File.Exists(SSEMI.PropertiesPath))
                            {
                                SSEMI.PropertiesCache = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.Properties);
                                SSEMI.PropertiesFile = Path.Combine(SSEMI.PropertiesCache, $"{SSEMI.LibrarySelected}{SSEMI.PropertiesType}");
                                SSEMI.WatcherFile = Path.Combine(SSEMI.PropertiesCache, $"*.{SSEMI.LibrarySelected}{SSEMI.PropertiesType}");

                                if (!Directory.Exists(SSEMI.PropertiesCache))
                                {
                                    Directory.CreateDirectory(SSEMI.PropertiesCache);
                                }

                                if (!File.Exists(SSEMI.PropertiesFile))
                                {
                                    SSSHF.CopyBuffer(SSEMI.PropertiesPath, SSEMI.PropertiesFile);
                                }

                                try
                                {
                                    SSEMI.Properties = SSTHP.ReadJson(SSEMI.PropertiesFile);
                                }
                                catch (NotSupportedException Exception)
                                {
                                    SSSHF.Delete(SSEMI.PropertiesFile);

                                    throw new NotSupportedException(Exception.Message);
                                }
                                catch (Exception Exception)
                                {
                                    SSSHF.Delete(SSEMI.PropertiesFile);

                                    throw new Exception(Exception.Message, Exception.InnerException);
                                }

                                SSEMI.Properties.State = true;

                                SSEHP.Watcher(SSEMI.WatcherFile);
                            }

                            if (File.Exists(SSEMI.CompatiblePath))
                            {
                                SSEMI.Compatible = SSTHC.ReadJson(SSEMI.CompatiblePath);
                                SSEMI.Compatible.State = true;
                            }

                            LocalServer = new(Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected));

                            SSEMI.Host = LocalServer.Host();

                            await LocalServer.StartAsync();

                            SSECSMI.CefEngine = new()
                            {
                                UseLayoutRounding = true,
                                SnapsToDevicePixels = true,
                                VerticalAlignment = SWVA.Stretch,
                                HorizontalAlignment = SWHA.Stretch
                            };

                            switch (SSEMI.Info.Type)
                            {
                                case SSDEWT.Gif:
                                    SSECSVG Gif = new();
                                    Gif.Show();
                                    break;
                                case SSDEWT.Url:
                                    SSECSVU Url = new();
                                    Url.Show();
                                    break;
                                case SSDEWT.Web:
                                    SSECSVW Web = new();
                                    Web.Show();
                                    break;
                                case SSDEWT.Video:
                                    SSECSVV Video = new();
                                    Video.Show();
                                    break;
                                case SSDEWT.YouTube:
                                    SSECSVYT YouTube = new();
                                    YouTube.Show();
                                    break;
                                default:
                                    Close();
                                    break;
                            }
                        }
                        else
                        {
                            Close();
                        }
                    }
                    else
                    {
                        Close();
                    }
                }
                else
                {
                    Close();
                }
            }
            else
            {
                Close();
            }
        }

        protected async void Downloader()
        {
#if X64
            string Url = string.Format(SMMRU.VCRedist, "x64");
            string Setup = Path.Combine(SMMRP.Temp, $"VC_redist.x64.{Guid.NewGuid()}.exe");
#elif X86
            string Url = string.Format(SMMRU.VCRedist, "x86");
            string Setup = Path.Combine(SMMRP.Temp, $"VC_redist.x86.{Guid.NewGuid()}.exe");
#else
            string Url = string.Format(SMMRU.VCRedist, "arm64");
            string Setup = Path.Combine(SMMRP.Temp, $"VC_redist.arm64.{Guid.NewGuid()}.exe");
#endif

            try
            {
                HttpResponseMessage Response = await SSDMI.Client.GetAsync(Url);

                Response.EnsureSuccessStatusCode();

                using FileStream Stream = new(Setup, FileMode.Create, FileAccess.Write, FileShare.None);

                await Response.Content.CopyToAsync(Stream);

                await Stream.FlushAsync();
                Stream.Close();

                Process Installer = new()
                {
                    StartInfo = new ProcessStartInfo()
                    {
                        UseShellExecute = true,
                        FileName = Setup
                    }
                };

                Installer.Start();

                Installer.WaitForExit();

                await Task.Delay(1500);

                Checker();
            }
            catch (Exception Exception)
            {
                SSWHD.Add("Downloader Exception", new Hashtable()
                {
                    { "Message", Exception.Message },
                    { "Inner Exception", Exception.InnerException?.Message }
                });

                Checker();
            }
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            Cef.Shutdown();

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SRHR.SetLanguage(SMMG.Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (SSSHI.Basic(SMMRM.Live, SMMRA.CefSharpLive) && SSEHR.Check())
            {
                SSSHS.Apply();

                if (SSSHC.Check())
                {
                    SSSHC.Change();
                }
                else
                {
                    Checker();
                }
            }
            else
            {
                Close();
            }
        }
    }
}