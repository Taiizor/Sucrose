using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using System.Globalization;
using System.IO;
using System.Windows;
using Application = System.Windows.Application;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SRER = Sucrose.Resources.Extension.Resources;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHOS = Sucrose.Shared.Core.Helper.OperatingSystem;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;
using SSSHE = Sucrose.Shared.Space.Helper.Exceptioner;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SSSHUE = Sucrose.Shared.Space.Helper.Unique;
using SSSHUR = Sucrose.Shared.Space.Helper.User;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSSHWE = Sucrose.Shared.Space.Helper.WatchException;
using SSSMDD = Sucrose.Shared.Space.Model.DiagnosticsData;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWHSI = Skylark.Wing.Helper.SystemInfo;
using SWNM = Skylark.Wing.Native.Methods;
using SMMCU = Sucrose.Memory.Manage.Constant.Update;
using SMMCC = Sucrose.Memory.Manage.Constant.Cycling;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;
using SSDMMU = Sucrose.Shared.Dependency.Manage.Manager.Update;
using SSDMMB = Sucrose.Shared.Dependency.Manage.Manager.Backgroundog;
using SSDMME = Sucrose.Shared.Dependency.Manage.Manager.Engine;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;
using SSDMMG = Sucrose.Shared.Dependency.Manage.Manager.General;
using SSDMMC = Sucrose.Shared.Dependency.Manage.Manager.Cycling;
using SMMRM = Sucrose.Memory.Manage.Readonly.Mutex;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SWVDEMB = Sucrose.Watchdog.View.DarkErrorMessageBox;
using SWVLEMB = Sucrose.Watchdog.View.LightErrorMessageBox;

namespace Sucrose.Watchdog
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static bool HasError { get; set; } = true;

        public App()
        {
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

                string Path = SMMI.WatchdogLogManager.LogFile();

                SSSHW.Start(SMMRA.Watchdog, Exception, Path);

                Close();
            }
        }

        protected void Configure(string[] Args)
        {
            if (Args.Any())
            {
                string Decode = SSECCE.BaseToText(Args.First());
                string[] Arguments = Decode.Split(SMR.ValueSeparatorChar);

                if (Arguments.Any() && (Arguments.Count() == 3 || Arguments.Count() == 5))
                {
                    Guid Id = Guid.NewGuid();
                    string Log = Arguments[2];
                    string Name = SSSHUR.GetName();
                    string Model = SSSHUR.GetModel();
                    string Application = Arguments[0];
                    Guid AppId = SSSHUE.Generate(Application);
                    string Manufacturer = SSSHUR.GetManufacturer();
                    CultureInfo Culture = new(SWNM.GetUserDefaultUILanguage());
                    string Text = Arguments.Count() == 5 ? Arguments[4] : string.Empty;
                    string Source = Arguments.Count() == 5 ? Arguments[3] : string.Empty;
                    string Message = SSSHE.GetMessage(SSSHWE.Convert(Arguments[1]), SRER.GetValue("Watchdog", "ErrorEmpty"), SMR.ExceptionSplit);

                    SSSMDD DiagnosticsData = new(Id, SSSHUE.Generate($"{Name}-{Model}-{Manufacturer}"), Application, AppId, Name, Model, SSCHOS.GetServer(), SMMM.Culture.ToUpperInvariant(), SSCHV.GetText(), SSCHF.GetName(), JObject.Parse(Arguments[1]), SSCHOS.GetWorkstation(), Culture.Name, SSCHA.GetText(), Manufacturer, Culture.NativeName, SSCHOS.GetText(), SSCHOS.GetProcessArchitectureText(), SSCHV.GetOSText(), SSCHOS.GetProcessorArchitecture(), SWHSI.GetSystemInfoArchitecture());

                    SSSHW.Write(Path.Combine(SMMRP.ApplicationData, SMR.AppName, SMR.CacheFolder, SMR.ReportFolder, $"{Id}.json"), JsonConvert.SerializeObject(DiagnosticsData, Formatting.Indented));

                    if (Application != SMMRA.Watchdog)
                    {
                        SSSHP.Kill(Application);
                    }

                    switch (SSDMMG.ThemeType)
                    {
                        case SEWTT.Dark:
                            SWVDEMB DarkMessageBox = new(Message, Log, Source, Text);
                            DarkMessageBox.ShowDialog();
                            break;
                        default:
                            SWVLEMB LightMessageBox = new(Message, Log, Source, Text);
                            LightMessageBox.ShowDialog();
                            break;
                    }

                    Close();
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

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnLastWindowClose;

            Configure(e.Args);
        }
    }
}