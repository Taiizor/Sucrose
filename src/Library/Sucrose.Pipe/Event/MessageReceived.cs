using Skylark.Standard.Extension.Cryptology;

namespace Sucrose.Pipe.Event
{
    public class MessageReceivedEventArgs : EventArgs
    {
        private string Data;

        public string Message
        {
            get
            {
                if (string.IsNullOrEmpty(Data))
                {
                    return string.Empty;
                }
                else
                {
                    return CryptologyExtension.BaseToText(Data);
                }
            }
            set
            {
                if (string.IsNullOrEmpty(value))
                {
                    Data = string.Empty;
                }
                else
                {
                    Data = CryptologyExtension.TextToBase(value);
                }
            }
        }
    }
}