using Grpc.Core;
using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.Greeter;

namespace Sucrose.Common.Services
{
    public class GreeterServerService : GreeterBase
    {
        public override Task<GreetingResponse> SayHello(GreetingRequest Request, ServerCallContext Context)
        {
            string Result = $"Author is {Request.Name}!";

            return Task.FromResult(new GreetingResponse { Message = Result });
        }
    }
}