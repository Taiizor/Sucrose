using CefSharp;
using CefSharp.Wpf.HwndHost;
using Microsoft.Win32;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Net.Http;
using System.Windows;
using Application = System.Windows.Application;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SHV = Skylark.Helper.Versionly;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SRER = Sucrose.Resources.Extension.Resources;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DialogType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSECSVG = Sucrose.Shared.Engine.CefSharp.View.Gif;
using SSECSVU = Sucrose.Shared.Engine.CefSharp.View.Url;
using SSECSVV = Sucrose.Shared.Engine.CefSharp.View.Video;
using SSECSVW = Sucrose.Shared.Engine.CefSharp.View.Web;
using SSECSVYT = Sucrose.Shared.Engine.CefSharp.View.YouTube;
using SSEHC = Sucrose.Shared.Engine.Helper.Cycyling;
using SSEHP = Sucrose.Shared.Engine.Helper.Properties;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEVDMB = Sucrose.Shared.Engine.View.DarkMessageBox;
using SSEVLMB = Sucrose.Shared.Engine.View.LightMessageBox;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSHC = Sucrose.Shared.Space.Helper.Cycyling;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSTHC = Sucrose.Shared.Theme.Helper.Compatible;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Live.CefSharp
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

                await SSWW.Watch_ThreadException(Exception);

                //Close();
                Message(Exception);
            };

            AppDomain.CurrentDomain.FirstChanceException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_FirstChanceException(Exception);

                //Close();
                //Message(Exception);
            };

            AppDomain.CurrentDomain.UnhandledException += async (s, e) =>
            {
                Exception Exception = (Exception)e.ExceptionObject;

                await SSWW.Watch_GlobalUnhandledException(Exception);

                //Close();
                Message(Exception);
            };

            TaskScheduler.UnobservedTaskException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_UnobservedTaskException(Exception);

                e.SetObserved();

                //Close();
                Message(Exception);
            };

            Current.DispatcherUnhandledException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_DispatcherUnhandledException(Exception);

                e.Handled = true;

                //Close();
                Message(Exception);
            };

            SHC.All = new CultureInfo(SMMM.Culture, true);
        }

        protected void Close()
        {
            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Message(Exception Exception)
        {
            if (HasError)
            {
                HasError = false;

                string Path = SMMI.CefSharpLiveLogManager.LogFile();

                SSSHW.Start(SMR.CefSharpLive, Exception, Path);

                Close();
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

                    if (Version != null && Installed != null && (int)Installed == 1)
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
            catch
            {
                return false;
            }
        }

        protected void Checker()
        {
            if (Check() || SMMM.CefSharpTime > DateTime.Now)
            {
                Configure();
            }
            else
            {
                SSDEDT DialogResult;

                bool Check = SMMM.CefsharpContinue;

                string CloseText = SRER.GetValue("Live", "Close");
                string ContinueText = SRER.GetValue("Live", "Continue");
                string DownloadText = SRER.GetValue("Live", "Download");
                string RememberText = SRER.GetValue("Live", "Remember");

                string DialogInfo = SRER.GetValue("Live", "Info", "CefSharp");
                string DialogMessage = SRER.GetValue("Live", "Message", "CefSharp");

                string DialogTitle = string.Format(SRER.GetValue("Live", "Title"), "CefSharp");

                switch (SSDMM.ThemeType)
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

                switch (DialogResult)
                {
                    case SSDEDT.Continue:
                        Configure();
                        break;
                    case SSDEDT.Download:
                        SMMI.UserSettingManager.SetSetting(SMC.CefsharpContinue, true);
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
            SMMI.UserSettingManager.SetSetting(SMC.CefSharpTime, DateTime.Now.AddDays(1));

            Configure();
        }

        protected void Configure()
        {
            SSEMI.LibraryLocation = SMMM.LibraryLocation;
            SSEMI.LibrarySelected = SMMM.LibrarySelected;

            if (SMMI.LibrarySettingManager.CheckFile() && !string.IsNullOrEmpty(SSEMI.LibrarySelected))
            {
                SSEMI.InfoPath = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, SMR.SucroseInfo);
                SSEMI.CompatiblePath = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, SMR.SucroseCompatible);
                SSEMI.PropertiesPath = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, SMR.SucroseProperties);

                if (File.Exists(SSEMI.InfoPath) && SSTHI.CheckJson(SSTHI.ReadInfo(SSEMI.InfoPath)))
                {
                    SSEMI.Info = SSTHI.ReadJson(SSEMI.InfoPath);

                    if (SSEMI.Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                    {
                        SSLHK.StopSubprocess();

#if NET48_OR_GREATER && DEBUG
                        CefRuntime.SubscribeAnyCpuAssemblyResolver();
#endif

                        CefSettings Settings = new()
                        {
                            UserAgent = SMMM.UserAgent,
                            CachePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.CefSharp)
                        };

                        SSEMI.BrowserSettings.CefSharp = SMMM.CefArguments;

                        if (!SSEMI.BrowserSettings.CefSharp.Any())
                        {
                            SSEMI.BrowserSettings.CefSharp = SSEMI.CefArguments;

                            SMMI.EngineSettingManager.SetSetting(SMC.CefArguments, SSEMI.BrowserSettings.CefSharp);
                        }

                        foreach (KeyValuePair<string, string> Argument in SSEMI.BrowserSettings.CefSharp)
                        {
                            Settings.CefCommandLineArgs.Add(Argument.Key, Argument.Value);
                        }

                        if (SMMM.DeveloperPort > 0)
                        {
                            Settings.RemoteDebuggingPort = SMMM.DeveloperPort;

                            if (!Settings.CefCommandLineArgs.ContainsKey("remote-allow-origins"))
                            {
                                Settings.CefCommandLineArgs.Add("remote-allow-origins", "*");
                            }
                        }

                        //Example of checking if a call to Cef.Initialize has already been made, we require this for
                        //our .Net 5.0 Single File Publish example, you don't typically need to perform this check
                        //if you call Cef.Initialze within your WPF App constructor.
                        if (!Cef.IsInitialized)
                        {
                            //Perform dependency check to make sure all relevant resources are in our output directory.
                            Cef.Initialize(Settings, performDependencyCheck: true, browserProcessHandler: null);
                        }

                        string Source = SSEMI.Info.Source;

                        if (!SSTHV.IsUrl(Source))
                        {
                            Source = Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected, Source);
                        }

                        SMMI.BackgroundogSettingManager.SetSetting(new KeyValuePair<string, bool>[]
                        {
                            new(SMC.PipeRequired, false),
                            new(SMC.AudioRequired, false),
                            new(SMC.SignalRequired, false),
                            new(SMC.PausePerformance, false)
                        });

                        if (SSTHV.IsUrl(Source) || File.Exists(Source))
                        {
                            SSSHS.Apply();

                            SSEHC.Start();

                            if (File.Exists(SSEMI.PropertiesPath))
                            {
                                SSEMI.PropertiesCache = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Properties);
                                SSEMI.PropertiesFile = Path.Combine(SSEMI.PropertiesCache, $"{SSEMI.LibrarySelected}.json");
                                SSEMI.WatcherFile = Path.Combine(SSEMI.PropertiesCache, $"*.{SSEMI.LibrarySelected}.json");

                                if (!Directory.Exists(SSEMI.PropertiesCache))
                                {
                                    Directory.CreateDirectory(SSEMI.PropertiesCache);
                                }

                                if (!File.Exists(SSEMI.PropertiesFile))
                                {
                                    File.Copy(SSEMI.PropertiesPath, SSEMI.PropertiesFile, true);
                                }

                                try
                                {
                                    SSEMI.Properties = SSTHP.ReadJson(SSEMI.PropertiesFile);
                                }
                                catch (NotSupportedException Ex)
                                {
                                    File.Delete(SSEMI.PropertiesFile);

                                    throw new NotSupportedException(Ex.Message);
                                }
                                catch (Exception Ex)
                                {
                                    File.Delete(SSEMI.PropertiesFile);

                                    throw new Exception(Ex.Message, Ex.InnerException);
                                }

                                SSEMI.Properties.State = true;

                                SSEHP.Watcher(SSEMI.WatcherFile);
                            }

                            if (File.Exists(SSEMI.CompatiblePath))
                            {
                                SSEMI.Compatible = SSTHC.ReadJson(SSEMI.CompatiblePath);
                                SSEMI.Compatible.State = true;
                            }

                            switch (SSEMI.Info.Type)
                            {
                                case SSDEWT.Gif:
                                    SSECSVG Gif = new(Source);
                                    Gif.Show();
                                    break;
                                case SSDEWT.Url:
                                    SSECSVU Url = new(Source);
                                    Url.Show();
                                    break;
                                case SSDEWT.Web:
                                    SSECSVW Web = new(Source);
                                    Web.Show();
                                    break;
                                case SSDEWT.Video:
                                    SSECSVV Video = new(Source);
                                    Video.Show();
                                    break;
                                case SSDEWT.YouTube:
                                    SSECSVYT YouTube = new(Source);
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
            string Url = "https://aka.ms/vs/17/release/vc_redist.x64.exe";
            string File = Path.Combine(Path.GetTempPath(), $"VC_redist.x64.{Guid.NewGuid()}.exe");
#elif X86
            string Url = "https://aka.ms/vs/17/release/vc_redist.x86.exe";
            string File = Path.Combine(Path.GetTempPath(), $"VC_redist.x86.{Guid.NewGuid()}.exe");
#else
            string Url = "https://aka.ms/vs/17/release/vc_redist.arm64.exe";
            string File = Path.Combine(Path.GetTempPath(), $"VC_redist.arm64.{Guid.NewGuid()}.exe");
#endif

            HttpClient Client = new();

            Client.DefaultRequestHeaders.Add("User-Agent", SMMM.UserAgent);

            HttpResponseMessage Response = await Client.GetAsync(Url);

            Response.EnsureSuccessStatusCode();

            using FileStream Stream = new(File, FileMode.Create, FileAccess.Write, FileShare.None);

            await Response.Content.CopyToAsync(Stream);

            await Stream.FlushAsync();
            Stream.Close();

            Process Installer = new()
            {
                StartInfo = new ProcessStartInfo()
                {
                    FileName = File,
                    UseShellExecute = true
                }
            };

            Installer.Start();

            Installer.WaitForExit();

            await Task.Delay(1500);

            Checker();
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

            SRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (SSSHI.Basic(SMR.LiveMutex, SMR.CefSharpLive) && SSEHR.Check())
            {
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