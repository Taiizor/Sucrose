using Newtonsoft.Json;
using SMR = Sucrose.Memory.Readonly;
using SSDESST = Sucrose.Shared.Dependency.Enum.StoreServerType;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSSIR = Sucrose.Shared.Store.Interface.Root;
using SMMRGU = Sucrose.Memory.Manage.Readonly.Url;

namespace Sucrose.Shared.Store.Helper
{
    internal static class Store
    {
        public static string Source(SSDESST Store)
        {
            return Store switch
            {
                SSDESST.GitHub => SMMRGU.RawGitHubStoreBranch,
                _ => SMR.SoferityStore,
            };
        }

        public static string Json(string Store)
        {
            string Content = SSSHF.Read(Store);

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