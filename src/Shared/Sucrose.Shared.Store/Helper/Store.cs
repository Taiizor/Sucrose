using Newtonsoft.Json;
using System.IO;
using SSSIR = Sucrose.Shared.Store.Interface.Root;

namespace Sucrose.Shared.Store.Helper
{
    internal static class Store
    {
        public static string Json(string Store)
        {
            return File.ReadAllText(Store);
        }

        public static SSSIR DeserializeRoot(string Store)
        {
            return JsonConvert.DeserializeObject<SSSIR>(Json(Store));
        }
    }
}