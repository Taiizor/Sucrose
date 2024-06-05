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
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
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
                    NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.DirectoryName,
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
                using HttpClient Client = new();

                HttpResponseMessage Response = new();

                Client.DefaultRequestHeaders.Add("User-Agent", SMMM.UserAgent);

                try
                {
                    Response = await Client.GetAsync($"{SMR.SoferityWebsite}/{SMR.SoferityReport}/{SMR.Online}/{SSSHU.GetGuid()}/{SRMI.InitializeTime / 1000}");
                }
                catch (Exception Exception)
                {
                    await SSWW.Watch_CatchException(Exception);
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
                using HttpClient Client = new();

                HttpResponseMessage Response = new();

                Client.DefaultRequestHeaders.Add("User-Agent", SMMM.UserAgent);

                try
                {
                    CultureInfo Culture = new(SWNM.GetUserDefaultUILanguage());

                    SSSMAD AnalyticsData = new(SMMM.Adult, SSSHU.GetName(), SSSHU.GetModel(), $"{SSDMM.StoreType}", SMMM.Startup, SMMM.Culture.ToUpperInvariant(), SSCHV.GetText(), SSCHF.GetName(), SSSHU.GetProcessor(), SSCHM.GetTotalMemory(), Culture.Name, SSSHU.GetNumberOfCores(), SSCHA.GetText(), SSSHU.GetManufacturer(), $"{SMMM.DisplayScreenType}", Culture.NativeName, SSCHOS.GetText(), SSCHOS.GetProcessArchitectureText(), SSCHV.GetOSText(), SSCHOS.GetProcessorArchitecture(), SWHSI.GetSystemInfoArchitecture());

                    StringContent Content = new(JsonConvert.SerializeObject(AnalyticsData, Formatting.Indented), Encoding.UTF8, "application/json");

                    Response = await Client.PostAsync($"{SMR.SoferityWebsite}/{SMR.SoferityReport}/{SMR.Statistic}/{SSSHU.GetGuid()}", Content);
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

                        Response = await Client.PostAsync($"{SMR.SoferityWebsite}/{SMR.SoferityReport}/{SMR.Error}/{SSSHU.GetGuid()}", Content);
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