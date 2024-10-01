using Newtonsoft.Json;
using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSDESST = Sucrose.Shared.Dependency.Enum.StoreServerType;
using SSSIR = Sucrose.Shared.Store.Interface.Root;

namespace Sucrose.Shared.Store.Helper
{
    internal static class Store
    {
        public static string Source(SSDESST Store)
        {
            return Store switch
            {
                SSDESST.GitHub => SMR.GitHubRawWebsite,
                _ => SMR.SoferityRawWebsite,
            };
        }

        public static string Json(string Store)
        {
            string Content = File.ReadAllText(Store);

            return string.IsNullOrWhiteSpace(Content) ? string.Empty : Content;
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