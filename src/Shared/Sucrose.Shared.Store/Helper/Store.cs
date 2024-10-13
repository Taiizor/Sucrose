using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using SSDESST = Sucrose.Shared.Dependency.Enum.StoreServerType;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSSIS = Sucrose.Shared.Store.Interface.Store;
using SMMRGU = Sucrose.Memory.Manage.Readonly.Url;

namespace Sucrose.Shared.Store.Helper
{
    internal partial class Store
    {
        public static string Source(SSDESST Store)
        {
            return Store switch
            {
                SSDESST.GitHub => SMMRGU.RawGitHubStoreBranch,
                _ => SMMRGU.SoferityStore,
            };
        }
    }

    internal partial class Store
    {
        public static string Read(string Path)
        {
            string Content = SSSHF.Read(Path);

            return string.IsNullOrWhiteSpace(Content) ? string.Empty : Content;
        }

        public static SSSIS FromJson(string Json)
        {
            return JsonConvert.DeserializeObject<SSSIS>(Json, Converter.Settings);
        }

        public static bool FromCheck(string Json)
        {
            try
            {
                JsonConvert.DeserializeObject<SSSIS>(Json, Converter.Settings);

                JToken.Parse(Json);

                return true;
            }
            catch
            {
                return false;
            }
        }

        public static bool ReadCheck(string Path)
        {
            try
            {
                string Content = Read(Path);

                JsonConvert.DeserializeObject<SSSIS>(Content, Converter.Settings);

                JToken.Parse(Content);

                return true;
            }
            catch
            {
                return false;
            }
        }

        public static SSSIS ReadJson(string Path)
        {
            return JsonConvert.DeserializeObject<SSSIS>(Read(Path), Converter.Settings);
        }

        public static void Write(string Path, SSSIS Json)
        {
            SSSHF.Write(Path, JsonConvert.SerializeObject(Json, Converter.Settings));
        }
    }
}