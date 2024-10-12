﻿using System.IO.Pipes;
using SMR = Sucrose.Memory.Readonly;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;

namespace Sucrose.Pipe.Helper
{
    internal class PipeClient : IDisposable
    {
        private NamedPipeClientStream _pipeClient;
        private StreamWriter _writer;

        public bool IsConnected => _pipeClient.IsConnected;

        public void Start(string pipeName)
        {
            _pipeClient = new(SMMRG.PipeServerName, pipeName, PipeDirection.Out);

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
                _writer = new(_pipeClient);

                _writer.WriteLine(message);
                _writer.Flush();
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