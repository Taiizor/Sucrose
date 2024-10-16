using Newtonsoft.Json;
using System.Globalization;
using System.IO;
using System.Net.Http;
using System.Text;
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
using SSDMMU = Sucrose.Shared.Dependency.Manage.Manager.Update;
using SSDMMB = Sucrose.Shared.Dependency.Manage.Manager.Backgroundog;
using SSDMMC = Sucrose.Shared.Dependency.Manage.Manager.Cycling;
using SSDMME = Sucrose.Shared.Dependency.Manage.Manager.Engine;
using SSSMOTD = Sucrose.Shared.Space.Model.OnlineTelemetryData;
using SSDMMG = Sucrose.Shared.Dependency.Manage.Manager.General;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHU = Sucrose.Shared.Space.Helper.User;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSSMATD = Sucrose.Shared.Space.Model.AnalyticTelemetryData;
using SSSMAD = Sucrose.Shared.Space.Model.AnalyticsData;
using SSSMTED = Sucrose.Shared.Space.Model.ThrowExceptionData;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWHSI = Skylark.Wing.Helper.SystemInfo;
using SWNM = Skylark.Wing.Native.Methods;
using Skylark.Enum;
using Sucrose.Memory.Manage.Constant;

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

                    try
                    {
                        SSSMOTD OnlineData = new()
                        {
                            AppVersion = SSCHV.GetText(),
                            Time = SRMI.InitializeTime / 1000
                        };

                        StringContent Content = new(JsonConvert.SerializeObject(OnlineData, Formatting.Indented), SMMRS.Encoding, "application/json");

                        HttpResponseMessage Response = await Client.PostAsync($"{SMMRU.Soferity}/{SMMRS.Version}/{SMMRS.Telemetry}/{SMMRS.Online}/{SSSHU.GetGuid()}", Content);

                        Response.EnsureSuccessStatusCode();
                    }
                    catch (Exception Exception)
                    {
                        await SSWW.Watch_CatchException(Exception);
                    }
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

                        try
                        {
                            CultureInfo Culture = new(SWNM.GetUserDefaultUILanguage());

                            SSSMAD AnalyticsData = new(SMMG.ExceptionData, SMMG.TelemetryData, $"{SSDMME.InputModuleType}", SMME.LibraryStart, SMMP.StorePreview, SMME.VolumeActive, SMME.DeveloperPort, SSSHU.GetNumberOfCores(), SMMP.StoreDuration, SSCHA.GetText(), SMME.DeveloperMode, SMMD.MenuVisible, SMML.DeleteCorrupt, SSSHU.GetManufacturer(), SMME.VolumeDesktop, $"{SSDMMB.CommunicationType}", $"{SMME.DisplayScreenType}", SMML.DeleteConfirm, SMMP.LibraryPreview, SMMP.StorePagination, $"{SSDMMB.CpuPerformance}", $"{SSDMMB.GpuPerformance}", $"{SSDMMC.TransitionType}", SMMD.AdvertisingDelay, SMMP.BackgroundImage, SSCHOS.GetText(), SMMP.BackgroundOpacity, SMMP.LibraryPagination, $"{SSDMMB.FocusPerformance}", $"{SSDMMB.PausePerformanceType}", $"{SSDMMB.SaverPerformance}", SSCHOS.GetNumberOfProcessors(), $"{SSDMMP.BackgroundStretch}", $"{SSDMMB.MemoryPerformance}", SMMB.PerformanceCounter, $"{SSDMMB.RemotePerformance}", $"{SSDMMB.BatteryPerformance}", $"{SSDMMB.NetworkPerformance}", $"{SSDMMB.VirtualPerformance}", SSCHOS.GetProcessArchitectureText(), SSCHV.GetOSText(), $"{SSDMMB.FullscreenPerformance}", SSCHOS.GetProcessorArchitecture(), SWHSI.GetSystemInfoArchitecture());

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
                                ThemeType = $"{SSDMMG.ThemeType}",
                                WallpaperLoop = SMME.WallpaperLoop,
                                CultureDisplay = Culture.NativeName,
                                EngineYouTube = $"{SSDMME.YouTube}",
                                DiscordConnect = SMMH.DiscordConnect,
                                GraphicAdapter = SMMB.GraphicAdapter,
                                NetworkAdapter = SMMB.NetworkAdapter,
                                TotalMemory = SSCHM.GetTotalMemory(),
                                EngineInputType = $"{SMME.InputType}",
                                StretchType = $"{SSDMME.StretchType}",
                                EngineInputDesktop = SMME.InputDesktop,
                                UserIdentifier = SSSHU.GetIdentifier(),
                                WallpaperVolume = SMME.WallpaperVolume,
                                EngineScreenType = $"{SMME.ScreenType}",
                                IsWorkstation = SSCHOS.GetWorkstation(),
                                UserIdentifying = SSSHU.GetIdentifying(),
                                WallpaperShuffle = SMME.WallpaperShuffle,
                                UpdateModuleType = $"{SSDMMU.ModuleType}",
                                UpdateServerType = $"{SSDMMU.ServerType}",
                                AdvertisingActive = SMMD.AdvertisingActive,
                                CyclingTransitionTime = SMMC.TransitionTime,
                                EngineApplication = $"{SSDMME.Application}",
                                UpdateChannelType = $"{SSCMMU.ChannelType}",
                                CultureCode = SMMG.Culture.ToUpperInvariant(),
                                StoreServerType = $"{SSDMMP.StoreServerType}",
                                UpdateExtensionType = $"{SSCMMU.ExtensionType}",
                                GraphicAdapters = string.Join(",", SSSHU.GetGraphic()),
                                NetworkAdapters = string.Join(",", SSSHU.GetNetwork()),
                                DeviceProcessors = string.Join(",", SSSHU.GetProcessor()),
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
                        catch (Exception Exception)
                        {
                            await SSWW.Watch_CatchException(Exception);

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

                        try
                        {
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
                        catch (Exception Exception)
                        {
                            await SSWW.Watch_CatchException(Exception);
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