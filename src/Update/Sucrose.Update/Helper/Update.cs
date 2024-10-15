using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using System.Net.Http;
using SSIIR = Skylark.Standard.Interface.IReleases;

namespace Sucrose.Update.Helper
{
    internal static class Update
    {
        private static readonly TimeSpan Time = TimeSpan.FromMinutes(5);

        private static readonly Dictionary<string, CachedData> Cache = new();

        public static string Releases(string Uri, string UserAgent)
        {
            string[] Keys = { Uri, UserAgent };
            string Key = string.Join(",", Keys);

            if (Cache.TryGetValue(Key, out CachedData Data))
            {
                if (DateTime.Now - Data.Timestamp < Time)
                {
                    if (Data.Status)
                    {
                        return Data.Content;
                    }
                    else
                    {
                        throw new Exception(Data.Content);
                    }
                }
            }

            HttpClient Client = InitializeClient(UserAgent);

            HttpResponseMessage Response = Client.GetAsync(Uri).Result;

            string Result = Response.Content.ReadAsStringAsync().Result;

            Cache[Key] = new CachedData(Response.IsSuccessStatusCode, Result, DateTime.Now);

            Response.EnsureSuccessStatusCode();

            if (Response.IsSuccessStatusCode)
            {
                return Result;
            }
            else
            {
                throw new Exception(Result);
            }
        }

        public static async Task<string> ReleasesAsync(string Uri, string UserAgent)
        {
            return await Task.Run(() => Releases(Uri, UserAgent));
        }

        public static object ReleasesObject(string Uri, string UserAgent)
        {
            return JsonConvert.DeserializeObject(Releases(Uri, UserAgent));
        }

        public static async Task<object> ReleasesObjectAsync(string Uri, string UserAgent)
        {
            return await Task.Run(() => ReleasesObject(Uri, UserAgent));
        }

        public static JArray ReleasesJArray(string Uri, string UserAgent)
        {
            return JArray.Parse(Releases(Uri, UserAgent));
        }

        public static async Task<JArray> ReleasesJArrayAsync(string Uri, string UserAgent)
        {
            return await Task.Run(() => ReleasesJArray(Uri, UserAgent));
        }

        public static List<SSIIR> ReleasesList(string Uri, string UserAgent)
        {
            return JsonConvert.DeserializeObject<List<SSIIR>>(Releases(Uri, UserAgent));
        }

        public static async Task<List<SSIIR>> ReleasesListAsync(string Uri, string UserAgent)
        {
            return await Task.Run(() => ReleasesList(Uri, UserAgent));
        }

        public class CachedData(bool status, string content, DateTime timestamp)
        {
            public bool Status { get; } = status;

            public string Content { get; } = content;

            public DateTime Timestamp { get; } = timestamp;
        }

        private static HttpClient InitializeClient(string UserAgent)
        {
            HttpClient Client = new();

            Client.DefaultRequestHeaders.Clear();

            Client.DefaultRequestHeaders.Add("User-Agent", UserAgent);

            return Client;
        }
    }
}