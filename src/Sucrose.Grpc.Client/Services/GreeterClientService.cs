using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.Greeter;

namespace Sucrose.Grpc.Client.Services
{
    public static class GreeterClientService
    {
        public static GreetingResponse GetHello(GreeterClient Client, string Author)
        {
            GreetingRequest Request = new() { Name = Author };

            GreetingResponse Response = Client.SayHello(Request);

            return Response;
        }
    }
}