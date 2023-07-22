using System.Runtime.InteropServices;

namespace Sucrose.Shared.Engine.Vexana.Struct
{
    [StructLayout(LayoutKind.Sequential)]
    public struct Gif
    {
        public int Min;
        public int Max;
        public int Total;
        public List<string> List;
    }
}