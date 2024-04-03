namespace Sucrose.Pipe.Event
{
    public class MessageReceivedEventArgs : EventArgs
    {
        public string Message { get; set; }



        //private string Data;

        //public string Message
        //{
        //    get
        //    {
        //        if (string.IsNullOrEmpty(Data))
        //        {
        //            return string.Empty;
        //        }
        //        else
        //        {
        //            return CryptologyExtension.BaseToText(Data);
        //        }
        //    }
        //    set
        //    {
        //        if (string.IsNullOrEmpty(value))
        //        {
        //            Data = string.Empty;
        //        }
        //        else
        //        {
        //            Data = CryptologyExtension.TextToBase(value);
        //        }
        //    }
        //}




        //private byte[] Data;

        //public string Message
        //{
        //    get
        //    {
        //        if (Data != null && Data.Any())
        //        {
        //            return CryptologyExtension.BaseToText(DecompressionExtension.Decompress(Data, DecompressionType.Deflate, CompressionMode.Decompress).DecompressedData);
        //        }
        //        else
        //        {
        //            return string.Empty;
        //        }
        //    }
        //    set
        //    {
        //        if (!string.IsNullOrEmpty(value))
        //        {
        //            Data = CompressionExtension.Compress(CryptologyExtension.TextToBase(value), CompressionType.Deflate, CompressionLevel.Optimal).CompressedData;
        //        }
        //    }
        //}
    }
}