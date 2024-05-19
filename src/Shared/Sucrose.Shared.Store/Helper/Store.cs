using Newtonsoft.Json;
using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSDEST = Sucrose.Shared.Dependency.Enum.StoreType;
using SSSIR = Sucrose.Shared.Store.Interface.Root;

namespace Sucrose.Shared.Store.Helper
{
    internal static class Store
    {
        public static string Source(SSDEST Store)
        {
            return Store switch
            {
                SSDEST.GitHub => SMR.GitHubRawWebsite,
                _ => SMR.SoferityRawWebsite,
            };
        }

        public static string Json(string Store)
        {
            return File.ReadAllText(Store);
        }

        public static bool CheckRoot(string Store)
        {
            try
            {
                JsonConvert.DeserializeObject<SSSIR>(Json(Store));

                return true;
            }
            catch
            {
                return false;
            }
        }

        public static SSSIR DeserializeRoot(string Store)
        {
            return JsonConvert.DeserializeObject<SSSIR>(Json(Store));
        }
    }
}