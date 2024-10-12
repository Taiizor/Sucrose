using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace Sucrose.Manager.Helper
{
    internal static class Validator
    {
        public static bool Json(string Content)
        {
            try
            {
                JsonConvert.DeserializeObject(Content);

                JToken.Parse(Content);

                return true;
            }
            catch
            {
                return false;
            }
        }
    }
}