using SPEMREA = Sucrose.Pipe.Event.MessageReceivedEventArgs;
using SPHPC = Sucrose.Pipe.Helper.PipeClient;
using SPHPS = Sucrose.Pipe.Helper.PipeServer;

namespace Sucrose.Pipe
{
    public class PipeT(string PipeName)
    {
        private string PipeName = PipeName;

        private bool ClientStarted;
        private bool ServerStarted;

        private bool ClientStopped;
        private bool ServerStopped;

        private readonly SPHPC PC = new();
        private readonly SPHPS PS = new();

        public event EventHandler<SPEMREA> MessageReceived;

        public void StartClient()
        {
            if (!ClientStarted)
            {
                PC.Start(PipeName);

                ClientStarted = true;
            }
        }

        public void StartClient(string Message)
        {
            if (!ClientStarted)
            {
                PC.Start(PipeName);

                ClientStarted = true;
            }

            if (!ClientStopped)
            {
                if (!PC.IsConnected)
                {
                    PC.Dispose();

                    PC.Start(PipeName);
                }

                PC.SendMessage(Message);
            }
        }

        public void StartServer()
        {
            if (!ServerStarted)
            {
                PS.Start(PipeName, MessageReceived);

                ServerStarted = true;
            }

            while (!ServerStopped)
            {
                if (!PS.IsConnected)
                {
                    PS.Dispose();

                    PS.Start(PipeName, MessageReceived);
                }
            }
        }

        public void StopClient()
        {
            ClientStopped = true;

            PC.Stop();
        }

        public void StopServer()
        {
            ServerStopped = true;

            PS.Stop();
        }

        public void DisposeClient()
        {
            ClientStopped = true;

            PC.Stop();
            PC.Dispose();
        }

        public void DisposeServer()
        {
            ServerStopped = true;

            PS.Stop();
            PS.Dispose();
        }

        protected virtual void OnMessageReceived(SPEMREA e)
        {
            MessageReceived?.Invoke(this, e);
        }
    }
}