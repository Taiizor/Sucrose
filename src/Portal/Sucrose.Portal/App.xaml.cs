using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Sucrose.Portal.Dependency;
using System.Globalization;
using System.Windows;
using Wpf.Ui;
using Wpf.Ui.DependencyInjection;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SHC = Skylark.Helper.Culture;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPMAC = Sucrose.Portal.Models.AppConfig;
using SPSAHS = Sucrose.Portal.Services.ApplicationHostService;
using SPSCIW = Sucrose.Portal.Services.Contracts.IWindow;
using SPSWPS = Sucrose.Portal.Services.WindowsProviderService;
using SPVMPDSVM = Sucrose.Portal.ViewModels.Pages.DonateSettingViewModel;
using SPVMPGSVM = Sucrose.Portal.ViewModels.Pages.GeneralSettingViewModel;
using SPVMPLVM = Sucrose.Portal.ViewModels.Pages.LibraryViewModel;
using SPVMPOSVM = Sucrose.Portal.ViewModels.Pages.OtherSettingViewModel;
using SPVMPPESVM = Sucrose.Portal.ViewModels.Pages.PerformanceSettingViewModel;
using SPVMPPLSVM = Sucrose.Portal.ViewModels.Pages.PersonalSettingViewModel;
using SPVMPSSVM = Sucrose.Portal.ViewModels.Pages.SystemSettingViewModel;
using SPVMPSVM = Sucrose.Portal.ViewModels.Pages.StoreViewModel;
using SPVMPWSVM = Sucrose.Portal.ViewModels.Pages.WallpaperSettingViewModel;
using SPVMWMWVM = Sucrose.Portal.ViewModels.Windows.MainWindowViewModel;
using SPVPLP = Sucrose.Portal.Views.Pages.LibraryPage;
using SPVPSDSP = Sucrose.Portal.Views.Pages.Setting.DonateSettingPage;
using SPVPSGSP = Sucrose.Portal.Views.Pages.Setting.GeneralSettingPage;
using SPVPSOSP = Sucrose.Portal.Views.Pages.Setting.OtherSettingPage;
using SPVPSP = Sucrose.Portal.Views.Pages.StorePage;
using SPVPSPESP = Sucrose.Portal.Views.Pages.Setting.PerformanceSettingPage;
using SPVPSPLSP = Sucrose.Portal.Views.Pages.Setting.PersonalSettingPage;
using SPVPSSSP = Sucrose.Portal.Views.Pages.Setting.SystemSettingPage;
using SPVPSWSP = Sucrose.Portal.Views.Pages.Setting.WallpaperSettingPage;
using SPVWMW = Sucrose.Portal.Views.Windows.MainWindow;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Portal
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static bool HasError { get; set; } = true;

        // The .NET Generic Host provides dependency injection, configuration, logging, and other services.
        // https://docs.microsoft.com/dotnet/core/extensions/generic-host
        // https://docs.microsoft.com/dotnet/core/extensions/dependency-injection
        // https://docs.microsoft.com/dotnet/core/extensions/configuration
        // https://docs.microsoft.com/dotnet/core/extensions/logging
        private static readonly IHost _host = Host
            .CreateDefaultBuilder()
            .ConfigureAppConfiguration(configure =>
            {
                configure.SetBasePath(AppContext.BaseDirectory);
            })
            .ConfigureServices((context, services) =>
            {
                // Navigation
                services.AddNavigationViewPageProvider();

                // App Host
                services.AddHostedService<SPSAHS>();

                // Main window container with navigation
                services.AddSingleton<SPSCIW, SPVWMW>();
                services.AddSingleton<SPVMWMWVM>();
                services.AddSingleton<IThemeService, ThemeService>();
                services.AddSingleton<ITaskBarService, TaskBarService>();
                services.AddSingleton<ISnackbarService, SnackbarService>();
                services.AddSingleton<INavigationService, NavigationService>();
                services.AddSingleton<IContentDialogService, ContentDialogService>();
                services.AddSingleton<SPSWPS>();

                // Top-level pages
                services.AddTransient<SPVPLP>();
                services.AddTransient<SPVMPLVM>();

                services.AddTransient<SPVPSP>();
                services.AddTransient<SPVMPSVM>();

                services.AddTransient<SPVPSDSP>();
                services.AddTransient<SPVMPDSVM>();

                services.AddTransient<SPVPSGSP>();
                services.AddTransient<SPVMPGSVM>();

                services.AddTransient<SPVPSOSP>();
                services.AddTransient<SPVMPOSVM>();

                services.AddTransient<SPVPSPESP>();
                services.AddTransient<SPVMPPESVM>();

                services.AddTransient<SPVPSPLSP>();
                services.AddTransient<SPVMPPLSVM>();

                services.AddTransient<SPVPSSSP>();
                services.AddTransient<SPVMPSSVM>();

                services.AddTransient<SPVPSWSP>();
                services.AddTransient<SPVMPWSVM>();

                // All other pages and view models
                services.AddTransientFromNamespace("Sucrose.Portal.Views", SHA.Assemble(SEAT.Executing));
                services.AddTransientFromNamespace("Sucrose.Portal.ViewModels", SHA.Assemble(SEAT.Executing));

                // Configuration
                services.Configure<SPMAC>(context.Configuration.GetSection(nameof(SPMAC)));
            })
            .Build();

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

        /// <summary>
        /// Gets registered service.
        /// </summary>
        /// <typeparam name="T">Type of the service to get.</typeparam>
        /// <returns>Instance of the service or <see langword="null"/>.</returns>
        public static T GetService<T>() where T : class
        {
            return _host.Services.GetService(typeof(T)) as T ?? null;
        }

        /// <summary>
        /// Gets registered service.
        /// </summary>
        /// <typeparam name="T">Type of the service to get.</typeparam>
        /// <returns>Instance of the service or <see langword="null"/>.</returns>
        public static T GetRequiredService<T>() where T : class
        {
            return _host.Services.GetRequiredService<T>();
        }

        protected void Close()
        {
            _host.StopAsync().Wait();

            _host.Dispose();

            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Message(Exception Exception)
        {
            if (HasError)
            {
                HasError = false;

                string Path = SMMI.PortalLogManager.LogFile();

                SSSHW.Start(SMR.Portal, Exception, Path);

                Close();
            }
        }

        protected void Configure()
        {
            SSSHS.Apply();

            _host.Start();
        }

        /// <summary>
        /// Occurs when the application is closing.
        /// </summary>
        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            Close();
        }

        /// <summary>
        /// Occurs when the application is loading.
        /// </summary>
        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnLastWindowClose;

            if (SSSHI.Basic(SMR.PortalMutex, SMR.Portal))
            {
                Configure();
            }
            else
            {
                Close();
            }
        }
    }
}