#if BROWSER

using Sucrose.WPF.CS;
using System.IO;
using SPIW = Sucrose.Pipe.Interface.Websiter;
using SPMI = Sucrose.Pipe.Manage.Internal;

namespace Sucrose.Shared.Pipe.Services
{
    public static class WebsiterPipeService
    {
        public static void Handler(SPEMREA e)
        {
            if (e != null && !string.IsNullOrEmpty(e.Message))
            {
                SPIW Data = JsonConvert.DeserializeObject<SPIW>(e.Message);

                Variables.State = true;
                Variables.Uri = Data.Url;
                Variables.Hook = Data.Hook;
            }
        }
    }
}

#endif