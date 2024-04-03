using System.IO.Pipes;
using SPEMREA = Sucrose.Pipe.Event.MessageReceivedEventArgs;

namespace Sucrose.Pipe.Helper
{
    internal class PipeServer : IDisposable
    {
        private NamedPipeServerStream _pipeServer;
        private StreamReader _reader;

        public bool IsConnected => _pipeServer.IsConnected;

        public void Start(string pipeName, EventHandler<SPEMREA> eventHandler)
        {
            _pipeServer = new(pipeName, PipeDirection.In);

            _pipeServer.WaitForConnection();

            while (IsConnected)
            {
                _reader = new(_pipeServer);
                string message = _reader.ReadLine();

                if (!string.IsNullOrEmpty(message))
                {
                    eventHandler?.Invoke(this, new SPEMREA { Message = message });
                }
            }
        }

        public void Stop()
        {
            if (_pipeServer != null && IsConnected)
            {
                _pipeServer.Disconnect();
            }
        }

        public void Dispose()
        {
            if (_pipeServer != null)
            {
                _pipeServer.Dispose();
            }

            if (_reader != null)
            {
                _reader.Dispose();
            }
        }
    }
}