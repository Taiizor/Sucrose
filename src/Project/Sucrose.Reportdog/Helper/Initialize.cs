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
using SSDMMB = Sucrose.Shared.Dependency.Manage.Manager.Backgroundog;
using SSDMMC = Sucrose.Shared.Dependency.Manage.Manager.Cycling;
using SSDMME = Sucrose.Shared.Dependency.Manage.Manager.Engine;
using SSDMMG = Sucrose.Shared.Dependency.Manage.Manager.General;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHU = Sucrose.Shared.Space.Helper.User;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSSMAD = Sucrose.Shared.Space.Model.AnalyticsData;
using SSSMDD = Sucrose.Shared.Space.Model.DiagnosticsData;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWHSI = Skylark.Wing.Helper.SystemInfo;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Backgroundog.Helper
{
    internal class Initialize : IDisposable
    {
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
                        await PostError(Record);
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
                    await PostError(e.FullPath);
                };

                TimerCallback Callback = InitializeTimer_Callback;
                SRMI.InitializeTimer = new(Callback, null, 0, SRMI.InitializeTime);

                SRMI.Watcher.EnableRaisingEvents = true;

                await PostStatistic();
            }
        }

        private static async Task GetOnline()
        {
            try
            {
                if (SMMG.Statistics && SSSHN.GetHostEntry())
                {
                    using HttpClient Client = new();

                    HttpResponseMessage Response = new();

                    Client.DefaultRequestHeaders.Add("User-Agent", SMMG.UserAgent);

                    try
                    {
                        Response = await Client.GetAsync($"{SMMRU.Soferity}/{SMMRS.SoferityVersion}/{SMMRS.SoferityReport}/{SMMRS.SoferityOnline}/{SSSHU.GetGuid()}/{SRMI.InitializeTime / 1000}");
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

        private static async Task PostStatistic()
        {
            try
            {
                if (SMMG.Statistics)
                {
                    if (SSSHN.GetHostEntry())
                    {
                        using HttpClient Client = new();

                        HttpResponseMessage Response = new();

                        Client.DefaultRequestHeaders.Add("User-Agent", SMMG.UserAgent);

                        try
                        {
                            CultureInfo Culture = new(SWNM.GetUserDefaultUILanguage());

                            SSSMAD AnalyticsData = new(SMMG.AppExit, $"{SSDMME.GifEngine}", SMME.Loop, $"{SSDMME.UrlEngine}", $"{SSDMME.WebEngine}", SMMP.Adult, SSSHU.GetName(), SMME.Volume, $"{SMME.InputType}", SSSHU.GetModel(), SMMG.Report, SSCHOS.GetServer(), $"{SSDMMP.StoreServerType}", $"{SSDMMG.ThemeType}", $"{SSDMME.VideoEngine}", SMMG.Startup, SMMH.DiscordState, $"{SMME.ScreenType}", SMME.Shuffle, $"{SSCMMU.ExtensionType}", SMMG.Visible, $"{SSCMMU.ChannelType}", SMMG.Culture.ToUpperInvariant(), SMMC.Cycyling, string.Join(",", SSSHU.GetGraphic()), SMML.LibraryLocation, string.Join(",", SSSHU.GetNetwork()), $"{SSDMME.StretchType}", SSCHV.GetText(), $"{SSDMME.YouTubeEngine}", SMMU.Auto, SSCHF.GetName(), string.Join(",", SSSHU.GetProcessor()), SMMG.Statistics, SMME.StoreStart, SMMD.AdvertisingState, SSSHU.GetIdentifier(), SMML.LibraryMove, SSCHM.GetTotalMemory(), SSCHOS.GetWorkstation(), SMMC.CycylingTime, $"{SSDMME.ApplicationEngine}", Culture.Name, SSSHU.GetIdentifying(), SMME.InputDesktop, $"{SSDMME.InputModuleType}", SMME.LibraryStart, SMMP.StorePreview, SMME.VolumeActive, SMME.DeveloperPort, SSSHU.GetNumberOfCores(), SMMP.StoreDuration, SSCHA.GetText(), SMME.DeveloperMode, SMMD.DonateVisible, SMML.LibraryDelete, SSSHU.GetManufacturer(), SMME.VolumeDesktop, $"{SSDMMB.CommunicationType}", $"{SMME.DisplayScreenType}", SMML.LibraryConfirm, SMMP.LibraryPreview, SMMP.StorePagination, $"{SSDMMB.CpuPerformance}", Culture.NativeName, $"{SSDMMB.GpuPerformance}", SMMB.GraphicAdapter, SMMB.NetworkAdapter, $"{SSDMMC.TransitionCycleType}", SMMD.AdvertisingDelay, SMMP.BackgroundImage, SSCHOS.GetText(), SMMP.BackgroundOpacity, SMMP.LibraryPagination, $"{SSDMMB.FocusPerformance}", $"{SSDMMB.PausePerformanceType}", $"{SSDMMB.SaverPerformance}", SSCHOS.GetNumberOfProcessors(), $"{SSDMMP.BackgroundStretch}", $"{SSDMMB.MemoryPerformance}", SMMB.PerformanceCounter, $"{SSDMMB.RemotePerformance}", $"{SSDMMB.BatteryPerformance}", $"{SSDMMB.NetworkPerformance}", $"{SSDMMB.VirtualPerformance}", SSCHOS.GetProcessArchitectureText(), SSCHV.GetOSText(), $"{SSDMMB.FullscreenPerformance}", SSCHOS.GetProcessorArchitecture(), SWHSI.GetSystemInfoArchitecture());

                            StringContent Content = new(JsonConvert.SerializeObject(AnalyticsData, Formatting.Indented), Encoding.UTF8, "application/json");

                            Response = await Client.PostAsync($"{SMMRU.Soferity}/{SMMRS.SoferityVersion}/{SMMRS.SoferityReport}/{SMMRS.SoferityStatistic}/{SSSHU.GetGuid()}", Content);
                        }
                        catch (Exception Exception)
                        {
                            await SSWW.Watch_CatchException(Exception);

                            await Task.Delay(3000);

                            await PostStatistic();
                        }

                        if (!Response.IsSuccessStatusCode)
                        {
                            await Task.Delay(3000);

                            await PostStatistic();
                        }
                    }
                    else
                    {
                        await Task.Delay(3000);

                        await PostStatistic();
                    }
                }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                await Task.Delay(3000);

                await PostStatistic();
            }
        }

        private static async Task PostError(string Path)
        {
            try
            {
                if (SMMG.Report && SSSHN.GetHostEntry())
                {
                    await Task.Delay(50);

                    if (File.Exists(Path))
                    {
                        using HttpClient Client = new();

                        HttpResponseMessage Response = new();

                        Client.DefaultRequestHeaders.Add("User-Agent", SMMG.UserAgent);

                        try
                        {
                            SSSMDD DiagnosticsData = JsonConvert.DeserializeObject<SSSMDD>(SSSHW.Read(Path));

                            StringContent Content = new(JsonConvert.SerializeObject(DiagnosticsData, Formatting.Indented), Encoding.UTF8, "application/json");

                            Response = await Client.PostAsync($"{SMMRU.Soferity}/{SMMRS.SoferityVersion}/{SMMRS.SoferityReport}/{SMMRS.SoferityError}/{SSSHU.GetGuid()}", Content);
                        }
                        catch (Exception Exception)
                        {
                            await SSWW.Watch_CatchException(Exception);
                        }

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
            await GetOnline();
        }

        public void Stop()
        {
            if (SRMI.Watcher != null)
            {
                SRMI.Watcher.EnableRaisingEvents = false;
                SRMI.InitializeTimer.Dispose();
                SRMI.Watcher.Dispose();
                SRMI.Watcher = null;
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}