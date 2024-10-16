using Newtonsoft.Json;
using System.Globalization;
using System.IO;
using System.Net.Http;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMC = Sucrose.Manager.Manage.Cycling;
using SMMD = Sucrose.Manager.Manage.Donate;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMG = Sucrose.Manager.Manage.General;
using SMMH = Sucrose.Manager.Manage.Hook;
using SMML = Sucrose.Manager.Manage.Library;
using SMMP = Sucrose.Manager.Manage.Portal;
using SMMRS = Sucrose.Memory.Manage.Readonly.Soferity;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;
using SMMU = Sucrose.Manager.Manage.Update;
using SRMI = Sucrose.Reportdog.Manage.Internal;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHM = Sucrose.Shared.Core.Helper.Memory;
using SSCHOS = Sucrose.Shared.Core.Helper.OperatingSystem;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSCMMU = Sucrose.Shared.Core.Manage.Manager.Update;
using SSDMMB = Sucrose.Shared.Dependency.Manage.Manager.Backgroundog;
using SSDMMC = Sucrose.Shared.Dependency.Manage.Manager.Cycling;
using SSDMME = Sucrose.Shared.Dependency.Manage.Manager.Engine;
using SSDMMG = Sucrose.Shared.Dependency.Manage.Manager.General;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;
using SSDMMU = Sucrose.Shared.Dependency.Manage.Manager.Update;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHU = Sucrose.Shared.Space.Helper.User;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSSMATD = Sucrose.Shared.Space.Model.AnalyticTelemetryData;
using SSSMOTD = Sucrose.Shared.Space.Model.OnlineTelemetryData;
using SSSMTED = Sucrose.Shared.Space.Model.ThrowExceptionData;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWHSI = Skylark.Wing.Helper.SystemInfo;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Reportdog.Helper
{
    internal class Initialize : IDisposable
    {
        public async void Stop()
        {
            if (SRMI.Watcher != null)
            {
                SRMI.Watcher.EnableRaisingEvents = false;
                SRMI.InitializeTimer.Dispose();
                SRMI.Watcher.Dispose();
                SRMI.Watcher = null;
            }

            await Task.CompletedTask;
        }

        public async void Start()
        {
            if (SRMI.Watcher == null)
            {
                if (!Directory.Exists(SRMI.Source))
                {
                    Directory.CreateDirectory(SRMI.Source);
                }
                else
                {
                    string[] Files = Directory.GetFiles(SRMI.Source, "*.*", SearchOption.TopDirectoryOnly);

                    foreach (string Record in Files)
                    {
                        await SendThrow(Record);
                    }
                }

                SRMI.Watcher = new()
                {
                    NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.DirectoryName | NotifyFilters.CreationTime,
                    Path = SRMI.Source,
                    Filter = "*.*"
                };

                SRMI.Watcher.Created += async (s, e) =>
                {
                    await SendThrow(e.FullPath);
                };

                TimerCallback Callback = InitializeTimer_Callback;
                SRMI.InitializeTimer = new(Callback, null, 0, SRMI.InitializeTime);

                SRMI.Watcher.EnableRaisingEvents = true;

                await SendAnalytic();
            }
        }

        private static async Task SendOnline()
        {
            try
            {
                if (SMMG.TelemetryData && SSSHN.GetHostEntry())
                {
                    using HttpClient Client = new();

                    Client.DefaultRequestHeaders.Add("User-Agent", SMMG.UserAgent);

                    SSSMOTD OnlineData = new()
                    {
                        AppVersion = SSCHV.GetText(),
                        Time = SRMI.InitializeTime / 1000
                    };

                    StringContent Content = new(JsonConvert.SerializeObject(OnlineData, Formatting.Indented), SMMRS.Encoding, "application/json");

                    HttpResponseMessage Response = await Client.PostAsync($"{SMMRU.Soferity}/{SMMRS.Version}/{SMMRS.Telemetry}/{SMMRS.Online}/{SSSHU.GetGuid()}", Content);

                    Response.EnsureSuccessStatusCode();
                }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        private static async Task SendAnalytic()
        {
            try
            {
                if (SMMG.TelemetryData)
                {
                    if (SSSHN.GetHostEntry())
                    {
                        using HttpClient Client = new();

                        Client.DefaultRequestHeaders.Add("User-Agent", SMMG.UserAgent);

                        CultureInfo Culture = new(SWNM.GetUserDefaultUILanguage());

                        SSSMATD AnalyticData = new()
                        {
                            AppExit = SMMG.AppExit,
                            UpdateAuto = SMMU.Auto,
                            LibraryMove = SMML.Move,
                            CultureName = Culture.Name,
                            UserName = SSSHU.GetName(),
                            CyclingActive = SMMC.Active,
                            EngineGif = $"{SSDMME.Gif}",
                            EngineUrl = $"{SSDMME.Url}",
                            EngineWeb = $"{SSDMME.Web}",
                            AppVersion = SSCHV.GetText(),
                            AppVisible = SMMG.AppVisible,
                            RunStartup = SMMG.RunStartup,
                            StoreAdult = SMMP.StoreAdult,
                            StoreStart = SMME.StoreStart,
                            IsServer = SSCHOS.GetServer(),
                            AppFramework = SSCHF.GetName(),
                            DeviceModel = SSSHU.GetModel(),
                            EngineVideo = $"{SSDMME.Video}",
                            LibraryLocation = SMML.Location,
                            IsException = SMMG.ExceptionData,
                            IsTelemetry = SMMG.TelemetryData,
                            LibraryStart = SMME.LibraryStart,
                            StorePreview = SMMP.StorePreview,
                            VolumeSilent = SMME.VolumeSilent,
                            AppArchitecture = SSCHA.GetText(),
                            ThemeType = $"{SSDMMG.ThemeType}",
                            DeveloperMode = SMME.DeveloperMode,
                            DeveloperPort = SMME.DeveloperPort,
                            OperatingSystem = SSCHOS.GetText(),
                            StoreDuration = SMMP.StoreDuration,
                            WallpaperLoop = SMME.WallpaperLoop,
                            CultureDisplay = Culture.NativeName,
                            EngineYouTube = $"{SSDMME.YouTube}",
                            DiscordConnect = SMMH.DiscordConnect,
                            GraphicAdapter = SMMB.GraphicAdapter,
                            LibraryPreview = SMMP.LibraryPreview,
                            NetworkAdapter = SMMB.NetworkAdapter,
                            TotalMemory = SSCHM.GetTotalMemory(),
                            EngineInputType = $"{SMME.InputType}",
                            StretchType = $"{SSDMME.StretchType}",
                            BackgroundImage = SMMP.BackgroundImage,
                            EngineInputDesktop = SMME.InputDesktop,
                            StorePagination = SMMP.StorePagination,
                            UserIdentifier = SSSHU.GetIdentifier(),
                            WallpaperVolume = SMME.WallpaperVolume,
                            EngineScreenType = $"{SMME.ScreenType}",
                            IsWorkstation = SSCHOS.GetWorkstation(),
                            AdvertisingDelay = SMMD.AdvertisingDelay,
                            EngineVolumeDesktop = SMME.VolumeDesktop,
                            NumberOfCores = SSSHU.GetNumberOfCores(),
                            OperatingSystemBuild = SSCHV.GetOSText(),
                            UserIdentifying = SSSHU.GetIdentifying(),
                            WallpaperShuffle = SMME.WallpaperShuffle,
                            AdvertisingMenuVisible = SMMD.MenuVisible,
                            LibraryDeleteConfirm = SMML.DeleteConfirm,
                            LibraryDeleteCorrupt = SMML.DeleteCorrupt,
                            UpdateModuleType = $"{SSDMMU.ModuleType}",
                            UpdateServerType = $"{SSDMMU.ServerType}",
                            BackgroundOpacity = SMMP.BackgroundOpacity,
                            AdvertisingActive = SMMD.AdvertisingActive,
                            LibraryPagination = SMMP.LibraryPagination,
                            CpuPerformance = $"{SSDMMB.CpuPerformance}",
                            CyclingTransitionTime = SMMC.TransitionTime,
                            EngineApplication = $"{SSDMME.Application}",
                            GpuPerformance = $"{SSDMMB.GpuPerformance}",
                            ManufacturerBrand = SSSHU.GetManufacturer(),
                            UpdateChannelType = $"{SSCMMU.ChannelType}",
                            PerformanceCounter = SMMB.PerformanceCounter,
                            CultureCode = SMMG.Culture.ToUpperInvariant(),
                            InputModuleType = $"{SSDMME.InputModuleType}",
                            StoreServerType = $"{SSDMMP.StoreServerType}",
                            DisplayScreenType = $"{SMME.DisplayScreenType}",
                            FocusPerformance = $"{SSDMMB.FocusPerformance}",
                            SaverPerformance = $"{SSDMMB.SaverPerformance}",
                            UpdateExtensionType = $"{SSCMMU.ExtensionType}",
                            BackgroundStretch = $"{SSDMMP.BackgroundStretch}",
                            CommunicationType = $"{SSDMMB.CommunicationType}",
                            MemoryPerformance = $"{SSDMMB.MemoryPerformance}",
                            RemotePerformance = $"{SSDMMB.RemotePerformance}",
                            CyclingTransitionType = $"{SSDMMC.TransitionType}",
                            BatteryPerformance = $"{SSDMMB.BatteryPerformance}",
                            NetworkPerformance = $"{SSDMMB.NetworkPerformance}",
                            NumberOfProcessors = SSCHOS.GetNumberOfProcessors(),
                            PausePerformance = $"{SSDMMB.PausePerformanceType}",
                            VirtualPerformance = $"{SSDMMB.VirtualPerformance}",
                            GraphicAdapters = string.Join(",", SSSHU.GetGraphic()),
                            NetworkAdapters = string.Join(",", SSSHU.GetNetwork()),
                            VolumeSilentSensitivity = SMME.VolumeSilentSensitivity,
                            DeviceProcessors = string.Join(",", SSSHU.GetProcessor()),
                            FullscreenPerformance = $"{SSDMMB.FullscreenPerformance}",
                            ProcessArchitecture = SSCHOS.GetProcessArchitectureText(),
                            ProcessorArchitecture = SSCHOS.GetProcessorArchitecture(),
                            OperatingSystemArchitecture = SWHSI.GetSystemInfoArchitecture()
                        };

                        StringContent Content = new(JsonConvert.SerializeObject(AnalyticData, Formatting.Indented), SMMRS.Encoding, "application/json");

                        HttpResponseMessage Response = await Client.PostAsync($"{SMMRU.Soferity}/{SMMRS.Version}/{SMMRS.Telemetry}/{SMMRS.Analytic}/{SSSHU.GetGuid()}", Content);

                        Response.EnsureSuccessStatusCode();

                        if (!Response.IsSuccessStatusCode)
                        {
                            await Task.Delay(3000);

                            await SendAnalytic();
                        }
                    }
                    else
                    {
                        await Task.Delay(3000);

                        await SendAnalytic();
                    }
                }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                await Task.Delay(3000);

                await SendAnalytic();
            }
        }

        private static async Task SendThrow(string Path)
        {
            try
            {
                if (SMMG.ExceptionData && SSSHN.GetHostEntry())
                {
                    await Task.Delay(50);

                    if (File.Exists(Path))
                    {
                        using HttpClient Client = new();

                        Client.DefaultRequestHeaders.Add("User-Agent", SMMG.UserAgent);

                        SSSMTED ThrowData = JsonConvert.DeserializeObject<SSSMTED>(SSSHW.Read(Path));

                        StringContent Content = new(JsonConvert.SerializeObject(ThrowData, Formatting.Indented), SMMRS.Encoding, "application/json");

                        HttpResponseMessage Response = await Client.PostAsync($"{SMMRU.Soferity}/{SMMRS.Version}/{SMMRS.Exception}/{SMMRS.Throw}/{SSSHU.GetGuid()}", Content);

                        Response.EnsureSuccessStatusCode();

                        if (Response.IsSuccessStatusCode)
                        {
                            await Task.Delay(50);

                            File.Delete(Path);
                        }
                    }
                }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        private async void InitializeTimer_Callback(object State)
        {
            await SendOnline();
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}