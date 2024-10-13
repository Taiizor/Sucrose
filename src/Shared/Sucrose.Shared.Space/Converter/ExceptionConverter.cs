﻿using Newtonsoft.Json;
using System.Collections;
using System.Diagnostics;
using System.Reflection;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SSSHE = Sucrose.Shared.Space.Helper.Exceptioner;
using SSSISE = Sucrose.Shared.Space.Interface.SerializableException;
using SSSISFD = Sucrose.Shared.Space.Interface.StackFrameData;

namespace Sucrose.Shared.Space.Converter
{
    public class ExceptionConverter : JsonConverter<Exception>
    {
        public override void WriteJson(JsonWriter writer, Exception value, JsonSerializer serializer)
        {
            SSSISE serializableException = ConvertToSerializableException(value);

            serializer.Serialize(writer, serializableException);
        }

        private SSSISE ConvertToSerializableException(Exception exception)
        {
            if (exception == null)
            {
                return null;
            }

            return new SSSISE
            {
                Data = exception.Data,
                Source = exception.Source,
                Message = exception.Message,
                HResult = exception.HResult,
                HelpURL = exception.HelpLink,
                RawStackTrace = exception.StackTrace,
                StackTrace = ParseStackTrace(exception),
                ClassName = exception.GetType().FullName,
                InnerException = ConvertToSerializableException(exception.InnerException),
                FullMessage = SSSHE.GetMessage(exception, "Unfortunately.", SMMRG.ExceptionSplit)
            };
        }

        private List<SSSISFD> ParseStackTrace(Exception exception)
        {
            StackTrace stackTrace = new(exception, true);

            StackFrame[] frames = stackTrace.GetFrames();

            if (frames == null)
            {
                return null;
            }

            List<SSSISFD> frameList = new();

            if (frames.Any())
            {
                foreach (StackFrame frame in frames)
                {
                    frameList.Add(new SSSISFD
                    {
                        FileName = frame.GetFileName(),
                        Method = frame.GetMethod().ToString(),
                        LineNumber = frame.GetFileLineNumber(),
                        ColumnNumber = frame.GetFileColumnNumber()
                    });
                }
            }

            return frameList;
        }

        public override Exception ReadJson(JsonReader reader, Type objectType, Exception existingValue, bool hasExistingValue, JsonSerializer serializer)
        {
            SSSISE serializableException = serializer.Deserialize<SSSISE>(reader);

            return ConvertToException(serializableException);
        }

        private Exception ConvertToException(SSSISE serializableException)
        {
            if (serializableException == null)
            {
                return null;
            }

            Type exceptionType = Type.GetType(serializableException.ClassName);

            if (exceptionType == null)
            {
                exceptionType = typeof(Exception);
            }

            Exception exception = null;

            try
            {
                exception = (Exception)Activator.CreateInstance(exceptionType, serializableException.Message);
            }
            catch
            {
                exception = (Exception)Activator.CreateInstance(exceptionType);

                FieldInfo messageField = typeof(Exception).GetField("_message", BindingFlags.NonPublic | BindingFlags.Instance);

                messageField?.SetValue(exception, serializableException.Message);
            }

            exception.HelpLink = serializableException.HelpURL;

            if (serializableException.Data != null && serializableException.Data.Count > 0)
            {
                if (exception.Data == null)
                {
                    FieldInfo dataField = typeof(Exception).GetField("_data", BindingFlags.NonPublic | BindingFlags.Instance);

                    dataField?.SetValue(exception, new Dictionary<object, object>());
                }

                foreach (DictionaryEntry entry in serializableException.Data)
                {
                    try
                    {
                        exception.Data.Add(JsonConvert.SerializeObject(entry.Key), JsonConvert.SerializeObject(entry.Value));
                    }
                    catch
                    {
                        try
                        {
                            exception.Data.Add(entry.Key, entry.Value);
                        }
                        catch
                        {
                            exception.Data.Add(entry.Key, null);
                        }
                    }
                }
            }

            if (serializableException.InnerException != null)
            {
                Exception innerException = ConvertToException(serializableException.InnerException);

                FieldInfo field = typeof(Exception).GetField("_innerException", BindingFlags.NonPublic | BindingFlags.Instance);

                field?.SetValue(exception, innerException);
            }

            return exception;
        }
    }
}