using Newtonsoft.Json;
using SSSCEC = Sucrose.Shared.Space.Converter.ExceptionConverter;
using SSSMTED = Sucrose.Shared.Space.Model.ThrowExceptionData;

namespace Sucrose.Shared.Space.Extension
{
    internal static class WatchException
    {
        public static Exception Convert(string Exception)
        {
            return JsonConvert.DeserializeObject<Exception>(Exception, new JsonSerializerSettings
            {
                ReferenceLoopHandling = ReferenceLoopHandling.Ignore,
                Formatting = Formatting.Indented,
                Converters =
                {
                    new SSSCEC()
                }
            });
        }

        public static string Convert(SSSMTED Data, Formatting Formatting = Formatting.Indented)
        {
            return JsonConvert.SerializeObject(Data, new JsonSerializerSettings
            {
                ReferenceLoopHandling = ReferenceLoopHandling.Ignore,
                Formatting = Formatting,
                Converters =
                {
                    new SSSCEC()
                }
            });
        }

        public static string Convert(Exception Exception, Formatting Formatting = Formatting.Indented)
        {
            return JsonConvert.SerializeObject(Exception, new JsonSerializerSettings
            {
                ReferenceLoopHandling = ReferenceLoopHandling.Ignore,
                Formatting = Formatting,
                Converters =
                {
                    new SSSCEC()
                }
            });
        }
    }
}