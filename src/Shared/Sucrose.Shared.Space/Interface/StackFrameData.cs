namespace Sucrose.Shared.Space.Interface
{
    public class StackFrameData
    {
        public string Method { get; set; }

        public int LineNumber { get; set; }

        public string FileName { get; set; }

        public int ColumnNumber { get; set; }
    }
}