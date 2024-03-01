package stages

import (
	"bytes"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"os"
	"path/filepath"
)

func UploadFile(instruction *SubmissionInstruction, fp string) (*OSSFile, error) {
	file, err := os.Open(fp)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	filename := filepath.Base(fp)

	var requestBody bytes.Buffer
	writer := multipart.NewWriter(&requestBody)

	ossKey := fmt.Sprintf("%s/%s", instruction.Dir, filename)
	if err := writer.WriteField("key", ossKey); err != nil {
		return nil, err
	}

	if err := writer.WriteField("OSSAccessKeyId", instruction.OSSAccessKeyId); err != nil {
		return nil, err
	}
	if err := writer.WriteField("Signature", instruction.Signature); err != nil {
		return nil, err
	}
	if err := writer.WriteField("policy", instruction.Policy); err != nil {
		return nil, err
	}

	part, err := writer.CreateFormFile("file", filename)
	if err != nil {
		return nil, err
	}
	_, err = io.Copy(part, file)
	if err != nil {
		fmt.Print(err)
		return nil, err
	}

	// Close the writer to finalize the multipart message
	err = writer.Close()
	if err != nil {
		return nil, err
	}

	request, err := http.NewRequest("POST", fmt.Sprintf("https://%s.oss-accelerate.aliyuncs.com", instruction.Bucket), &requestBody)
	if err != nil {
		return nil, err
	}

	request.Header.Set("Content-Type", writer.FormDataContentType())

	client := &http.Client{}
	response, err := client.Do(request)
	if err != nil {
		return nil, err
	}
	defer response.Body.Close()

	if response.StatusCode != http.StatusOK && response.StatusCode != http.StatusCreated && response.StatusCode != http.StatusNoContent {
		err := fmt.Errorf("bad status: %s", response.Status)
		// bodyBytes, _ := io.ReadAll(response.Body)
		// fmt.Println("Response Body:", string(bodyBytes))
		return nil, err
	}

	return &OSSFile{
		Key:    ossKey,
		Bucket: instruction.Bucket,
	}, nil
}
