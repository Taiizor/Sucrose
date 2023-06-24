#if BROWSER

using Grpc.Core;
using Sucrose.Grpc.Common;
using Sucrose.WPF.CS;
using static Sucrose.Grpc.Common.Websiter;

namespace Sucrose.Common.Services
{
    public class WebsiterServerService : WebsiterBase
    {
        public override Task<WebsiterChangeResponse> ChangeAddress(WebsiterChangeRequest Request, ServerCallContext Context)
        {
            Variables.State = true;
            Variables.Uri = Request.Url;
            Variables.Hook = Request.Hook;

            return Task.FromResult(new WebsiterChangeResponse { Result = true });
        }
    }
}

#endif