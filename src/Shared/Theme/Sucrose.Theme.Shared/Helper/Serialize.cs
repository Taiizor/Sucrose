using Newtonsoft.Json;

namespace Sucrose.Theme.Shared.Helper
{
    internal static class Serialize
    {
        public static string ToJson(this Info self)
        {
            return JsonConvert.SerializeObject(self, Converter.Settings);
        }

        public static string ToJson(this Properties self)
        {
            return JsonConvert.SerializeObject(self, Converter.Settings);
        }
    }
}