using Microsoft.Web.WebView2.Core;
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
using SSEHA = Sucrose.Shared.Engine.Helper.Awakening;
using SSEHCR = Sucrose.Shared.Engine.Helper.Crashing;
using SSEHCY = Sucrose.Shared.Engine.Helper.Cycyling;
using SSEHP = Sucrose.Shared.Engine.Helper.Properties;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEVDMB = Sucrose.Shared.Engine.View.DarkMessageBox;
using SSEVLMB = Sucrose.Shared.Engine.View.LightMessageBox;
using SSEWVHP = Sucrose.Shared.Engine.WebView.Helper.Properties;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSEWVVG = Sucrose.Shared.Engine.WebView.View.Gif;
using SSEWVVU = Sucrose.Shared.Engine.WebView.View.Url;
using SSEWVVV = Sucrose.Shared.Engine.WebView.View.Video;
using SSEWVVW = Sucrose.Shared.Engine.WebView.View.Web;
using SSEWVVYT = Sucrose.Shared.Engine.WebView.View.YouTube;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSHC = Sucrose.Shared.Space.Helper.Cycyling;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSTHC = Sucrose.Shared.Theme.Helper.Compatible;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;
using SSWHD = Sucrose.Shared.Watchdog.Helper.Dataset;
using SWHA = System.Windows.HorizontalAlignment;
using SWVA = System.Windows.VerticalAlignment;

namespace Sucrose.Live.WebView
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static bool HasError { get; set; } = true;

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

            SSDHR.Configure();

            SSDHG.Configure();
        }

        protected void Close()
        {
            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Message(Exception Exception, bool Show)
        {
            if (HasError)
            {
                HasError = !Show;

                string Path = SMMI.WebViewLiveLogManager.LogFile();

                SSSHW.Start(SMMRA.WebViewLive, Exception, Show, Path);

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
                string Version = CoreWebView2Environment.GetAvailableBrowserVersionString();

                SSWHD.Add("WebView Version", Version);

                if (string.IsNullOrEmpty(Version))
                {
                    return false;
                }
                else
                {
                    if (SHV.Compare(SHV.Clear(Version), SHV.Clear("131.0.2903.70")) != SEVT.Latest)
                    {
                        return true;
                    }
                    else
                    {
                        return false;
                    }
                }
            }
            catch (Exception Exception)
            {
                SSWHD.Add("WebView Check Exception", new Hashtable()
                {
                    { "Message", Exception.Message },
                    { "Inner Exception", Exception.InnerException?.Message }
                });

                return false;
            }
        }

        protected void Checker()
        {
            SSWHD.Add("WebView Checker", new Hashtable()
            {
                { "Check", Check() },
                { "WebView Time", SMMW.WebViewTime },
                { "WebView Time Check", SMMW.WebViewTime > DateTime.Now }
            });

            if (Check() || SMMW.WebViewTime > DateTime.Now)
            {
                Configure();
            }
            else
            {
                SSDEDT DialogResult;

                bool Check = SMMW.WebViewContinue;

                string CloseText = SRER.GetValue("Live", "Close");
                string ContinueText = SRER.GetValue("Live", "Continue");
                string DownloadText = SRER.GetValue("Live", "Download");
                string RememberText = SRER.GetValue("Live", "Remember");

                string DialogInfo = SRER.GetValue("Live", "Info", "WebView");
                string DialogMessage = SRER.GetValue("Live", "Message", "WebView");

                string DialogTitle = string.Format(SRER.GetValue("Live", "Title"), "WebView");

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

                SSWHD.Add("WebView Dialog Result", $"{DialogResult}");

                switch (DialogResult)
                {
                    case SSDEDT.Continue:
                        Configure();
                        break;
                    case SSDEDT.Download:
                        SMMI.WarehouseSettingManager.SetSetting(SMMCW.WebViewContinue, true);
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
            SMMI.WarehouseSettingManager.SetSetting(SMMCW.WebViewTime, DateTime.Now.AddDays(1));

            Configure();
        }

        protected void Configure()
        {
            SSEMI.LibraryLocation = SMML.Location;
            SSEMI.LibrarySelected = SMML.Selected;

            if (SMMI.LibrarySettingManager.CheckFile() && !string.IsNullOrEmpty(SSEMI.LibrarySelected))
            {
                SSEMI.InfoPath = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, SMMRC.SucroseInfo);
                SSEWVMI.WebPath = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.WebView2);
                SSEMI.CompatiblePath = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, SMMRC.SucroseCompatible);
                SSEMI.PropertiesPath = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, SMMRC.SucroseProperties);

                if (File.Exists(SSEMI.InfoPath) && SSTHI.ReadCheck(SSEMI.InfoPath))
                {
                    SSEMI.Info = SSTHI.ReadJson(SSEMI.InfoPath);

                    if (SSEMI.Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                    {
                        SSLHK.StopSubprocess();

                        CoreWebView2EnvironmentOptions Options = new()
                        {
                            Language = SMMG.Culture,
                            AreBrowserExtensionsEnabled = false,
                            IsCustomCrashReportingEnabled = false,
                            ScrollBarStyle = CoreWebView2ScrollbarStyle.FluentOverlay
                        };

                        SSEMI.BrowserSettings.WebView = SMME.WebArguments;

                        if (!SSEMI.BrowserSettings.WebView.Any())
                        {
                            SSEMI.BrowserSettings.WebView = SSEMI.WebArguments;

                            SMMI.EngineSettingManager.SetSetting(SMMCE.WebArguments, SSEMI.BrowserSettings.WebView);
                        }

                        Options.AdditionalBrowserArguments = string.Join(" ", SSEMI.BrowserSettings.WebView);

                        if (SMME.DeveloperPort > 0)
                        {
                            if (!Options.AdditionalBrowserArguments.Contains("--remote-debugging-port"))
                            {
                                Options.AdditionalBrowserArguments += $" --remote-debugging-port={SMME.DeveloperPort}";
                            }

                            if (!Options.AdditionalBrowserArguments.Contains("--remote-allow-origins"))
                            {
                                Options.AdditionalBrowserArguments += $" --remote-allow-origins=*";
                            }

                            if (!Options.AdditionalBrowserArguments.Contains("--enable-features=msEdgeDevToolsWdpRemoteDebugging"))
                            {
                                Options.AdditionalBrowserArguments += $" --enable-features=msEdgeDevToolsWdpRemoteDebugging";
                            }
                        }

                        if (SMME.HardwareAcceleration)
                        {
                            if (!Options.AdditionalBrowserArguments.Contains("--enable-gpu"))
                            {
                                Options.AdditionalBrowserArguments += $" --enable-gpu";
                            }

                            if (!Options.AdditionalBrowserArguments.Contains("--enable-gpu-vsync"))
                            {
                                Options.AdditionalBrowserArguments += $" --enable-gpu-vsync";
                            }
                        }
                        else
                        {
                            if (!Options.AdditionalBrowserArguments.Contains("--disable-gpu"))
                            {
                                Options.AdditionalBrowserArguments += $" --disable-gpu";
                            }

                            if (!Options.AdditionalBrowserArguments.Contains("--disable-gpu-vsync"))
                            {
                                Options.AdditionalBrowserArguments += $" --disable-gpu-vsync";
                            }
                        }

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
                            
                            if (!Options.AdditionalBrowserArguments.Contains("--proxy-server"))
                            {
                                Options.AdditionalBrowserArguments += $" --proxy-server=\"{proxyUrl}\"";
                            }
                        }

                        Task<CoreWebView2Environment> Environment = CoreWebView2Environment.CreateAsync(null, SSEWVMI.WebPath, Options);

                        SSEWVMI.WebEngine = new()
                        {
                            UseLayoutRounding = true,
                            SnapsToDevicePixels = true,
                            VerticalAlignment = SWVA.Stretch,
                            HorizontalAlignment = SWHA.Stretch,
                            DefaultBackgroundColor = Color.Black
                        };

                        SSEWVMI.WebEngine.EnsureCoreWebView2Async(Environment.Result);

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
                                SSEWVHP.Start();
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

                            SSEMI.Host = $"{Path.Combine("file:///", SSEMI.LibraryLocation, SSEMI.LibrarySelected)}/";

                            switch (SSEMI.Info.Type)
                            {
                                case SSDEWT.Gif:
                                    SSEWVVG Gif = new();
                                    Gif.Show();
                                    break;
                                case SSDEWT.Url:
                                    SSEWVVU Url = new();
                                    Url.Show();
                                    break;
                                case SSDEWT.Web:
                                    SSEWVVW Web = new();
                                    Web.Show();
                                    break;
                                case SSDEWT.Video:
                                    SSEWVVV Video = new();
                                    Video.Show();
                                    break;
                                case SSDEWT.YouTube:
                                    SSEWVVYT YouTube = new();
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
            string Setup = Path.Combine(SMMRP.Temp, $"MicrosoftEdgeWebView2Setup.{Guid.NewGuid()}.exe");

            try
            {
                HttpResponseMessage Response = await SSDMI.Client.GetAsync(SMMRU.WebView2);

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

            //

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SRHR.SetLanguage(SMMG.Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (SSSHI.Basic(SMMRM.Live, SMMRA.WebViewLive) && SSEHR.Check())
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