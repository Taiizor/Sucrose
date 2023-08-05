using Newtonsoft.Json;

namespace Sucrose.Shared.Theme.Helper
{
    internal static class Serialize
    {
        public static string ToJson(this Info Self)
        {
            return JsonConvert.SerializeObject(Self, Converter.Settings);
        }

        public static string ToJson(this Properties Self)
        {
            return JsonConvert.SerializeObject(Self, Converter.Settings);
        }
    }
}