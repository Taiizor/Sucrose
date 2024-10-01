using Newtonsoft.Json;
using System.Globalization;
using System.IO;
using System.Net.Http;
using System.Text;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SRMI = Sucrose.Reportdog.Manage.Internal;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHM = Sucrose.Shared.Core.Helper.Memory;
using SSCHOS = Sucrose.Shared.Core.Helper.OperatingSystem;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSCMM = Sucrose.Shared.Core.Manage.Manager;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
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
                if (SMMM.Statistics && SSSHN.GetHostEntry())
                {
                    using HttpClient Client = new();

                    HttpResponseMessage Response = new();

                    Client.DefaultRequestHeaders.Add("User-Agent", SMMM.UserAgent);

                    try
                    {
                        Response = await Client.GetAsync($"{SMR.SoferityWebsite}/{SMR.SoferityVersion}/{SMR.SoferityReport}/{SMR.SoferityOnline}/{SSSHU.GetGuid()}/{SRMI.InitializeTime / 1000}");
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
                if (SMMM.Statistics)
                {
                    if (SSSHN.GetHostEntry())
                    {
                        using HttpClient Client = new();

                        HttpResponseMessage Response = new();

                        Client.DefaultRequestHeaders.Add("User-Agent", SMMM.UserAgent);

                        try
                        {
                            CultureInfo Culture = new(SWNM.GetUserDefaultUILanguage());

                            SSSMAD AnalyticsData = new(SMMM.Exit, $"{SSDMM.GifEngine}", SMMM.Loop, $"{SSDMM.UrlEngine}", $"{SSDMM.WebEngine}", SMMM.Adult, SSSHU.GetName(), SMMM.Volume, $"{SMMM.InputType}", SSSHU.GetModel(), SMMM.Report, SSCHOS.GetServer(), $"{SSDMM.StoreType}", $"{SSDMM.ThemeType}", $"{SSDMM.VideoEngine}", SMMM.Startup, SMMM.DiscordState, $"{SMMM.ScreenType}", SMMM.Shuffle, $"{SSCMM.UpdateExtensionType}", SMMM.Visible, $"{SSCMM.UpdateChannelType}", SMMM.Culture.ToUpperInvariant(), SMMM.Cycyling, string.Join(",", SSSHU.GetGraphic()), SMMM.LibraryLocation, string.Join(",", SSSHU.GetNetwork()), $"{SSDMM.StretchType}", SSCHV.GetText(), $"{SSDMM.YouTubeEngine}", SMMM.AutoUpdate, SSCHF.GetName(), string.Join(",", SSSHU.GetProcessor()), SMMM.Statistics, SMMM.StoreStart, SMMM.AdvertisingState, SSSHU.GetIdentifier(), SMMM.LibraryMove, SSCHM.GetTotalMemory(), SSCHOS.GetWorkstation(), SMMM.CycylingTime, $"{SSDMM.ApplicationEngine}", Culture.Name, SSSHU.GetIdentifying(), SMMM.InputDesktop, $"{SSDMM.InputModuleType}", SMMM.LibraryStart, SMMM.StorePreview, SMMM.VolumeActive, SMMM.DeveloperPort, SSSHU.GetNumberOfCores(), SMMM.StoreDuration, SSCHA.GetText(), SMMM.DeveloperMode, SMMM.DonateVisible, SMMM.LibraryDelete, SSSHU.GetManufacturer(), SMMM.VolumeDesktop, $"{SSDMM.CommunicationType}", $"{SMMM.DisplayScreenType}", SMMM.LibraryConfirm, SMMM.LibraryPreview, SMMM.StorePagination, $"{SSDMM.CpuPerformance}", Culture.NativeName, $"{SSDMM.GpuPerformance}", SMMM.GraphicAdapter, SMMM.NetworkAdapter, $"{SSDMM.TransitionType}", SMMM.AdvertisingDelay, SMMM.BackgroundImage, SSCHOS.GetText(), SMMM.BackgroundOpacity, SMMM.LibraryPagination, $"{SSDMM.FocusPerformance}", $"{SSDMM.PausePerformanceType}", $"{SSDMM.SaverPerformance}", SSCHOS.GetNumberOfProcessors(), $"{SSDMM.BackgroundStretch}", $"{SSDMM.MemoryPerformance}", SMMM.PerformanceCounter, $"{SSDMM.RemotePerformance}", $"{SSDMM.BatteryPerformance}", $"{SSDMM.NetworkPerformance}", $"{SSDMM.VirtualPerformance}", SSCHOS.GetProcessArchitectureText(), SSCHV.GetOSText(), $"{SSDMM.FullscreenPerformance}", SSCHOS.GetProcessorArchitecture(), SWHSI.GetSystemInfoArchitecture());

                            StringContent Content = new(JsonConvert.SerializeObject(AnalyticsData, Formatting.Indented), Encoding.UTF8, "application/json");

                            Response = await Client.PostAsync($"{SMR.SoferityWebsite}/{SMR.SoferityVersion}/{SMR.SoferityReport}/{SMR.SoferityStatistic}/{SSSHU.GetGuid()}", Content);
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
                if (SMMM.Report && SSSHN.GetHostEntry())
                {
                    await Task.Delay(50);

                    if (File.Exists(Path))
                    {
                        using HttpClient Client = new();

                        HttpResponseMessage Response = new();

                        Client.DefaultRequestHeaders.Add("User-Agent", SMMM.UserAgent);

                        try
                        {
                            SSSMDD DiagnosticsData = JsonConvert.DeserializeObject<SSSMDD>(SSSHW.Read(Path));

                            StringContent Content = new(JsonConvert.SerializeObject(DiagnosticsData, Formatting.Indented), Encoding.UTF8, "application/json");

                            Response = await Client.PostAsync($"{SMR.SoferityWebsite}/{SMR.SoferityVersion}/{SMR.SoferityReport}/{SMR.SoferityError}/{SSSHU.GetGuid()}", Content);
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