#if WPF_CS

using Grpc.Core;
using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.Websiter;

namespace Sucrose.Common.Services
{
    public class WebsiterServerService : WebsiterBase
    {
        public override Task<WebsiterChangeResponse> ChangeAddress(WebsiterChangeRequest Request, ServerCallContext Context)
        {
            Sucrose.WPF.CS.Variables.State = true;
            Sucrose.WPF.CS.Variables.Uri = Request.Url;
            Sucrose.WPF.CS.Variables.Hook = Request.Hook;

            return Task.FromResult(new WebsiterChangeResponse { Result = true });
        }
    }
}

#endif