using Newtonsoft.Json;
using SSSIS = Sucrose.Shared.Store.Interface.Store;

namespace Sucrose.Shared.Store.Helper
{
    internal static class Serialize
    {
        public static string ToJson(this SSSIS Self)
        {
            return JsonConvert.SerializeObject(Self, Converter.Settings);
        }
    }
}