using System.IO.Pipes;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Pipe.Helper
{
    internal class PipeClient : IDisposable
    {
        private NamedPipeClientStream _pipeClient;
        private StreamWriter _writer;

        public bool IsConnected => _pipeClient.IsConnected;

        public void Start(string pipeName)
        {
            _pipeClient = new(SMR.PipeServerName, pipeName, PipeDirection.Out);

            _pipeClient.Connect();
        }

        public void Stop()
        {
            if (_pipeClient != null && IsConnected)
            {
                _pipeClient.Close();
            }
        }

        public void SendMessage(string message)
        {
            if (_pipeClient == null)
            {
                return;
            }

            if (!IsConnected)
            {
                return;
            }

            if (!string.IsNullOrEmpty(message))
            {
                StreamWriter writer = new(_pipeClient);

                writer.WriteLine(message);
                writer.Flush();
            }
        }

        public void Dispose()
        {
            if (_pipeClient != null)
            {
                _pipeClient.Dispose();
            }

            if (_writer != null)
            {
                _writer.Dispose();
            }
        }
    }
}