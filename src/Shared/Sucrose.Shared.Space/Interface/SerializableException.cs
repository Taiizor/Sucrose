using System.Collections;

namespace Sucrose.Shared.Space.Interface
{
    public class SerializableException
    {
        public int HResult { get; set; }

        public string Source { get; set; }

        public string HelpURL { get; set; }

        public string Message { get; set; }

        public string ClassName { get; set; }

        public IDictionary Data { get; set; }

        public List<StackFrameData> StackTrace { get; set; }

        public SerializableException InnerException { get; set; }
    }
}