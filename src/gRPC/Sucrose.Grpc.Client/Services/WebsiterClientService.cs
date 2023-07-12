using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.Websiter;

namespace Sucrose.Grpc.Client.Services
{
    public static class WebsiterClientService
    {
        public static WebsiterChangeResponse ChangeAddress(WebsiterClient Client, string Url, bool Hook)
        {
            WebsiterChangeRequest Request = new() { Url = Url, Hook = Hook };

            WebsiterChangeResponse Response = Client.ChangeAddress(Request);

            return Response;
        }
    }
}