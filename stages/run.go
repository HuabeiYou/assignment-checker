package stages

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
)

type OSSFile struct {
	Key    string `json:"key"`
	Bucket string `json:"bucket"`
}

type RunBody struct {
	TestEntry string    `json:"test_entry"`
	TestEnv   []OSSFile `json:"test_env"`
	Files     []OSSFile `json:"files"`
}

func RunTest(runnerLocation string, requestBody *RunBody) (string, error) {
	content, _ := json.Marshal(requestBody)
	resp, err := http.Post(runnerLocation, "application/json", bytes.NewReader(content))
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	var contentBuilder strings.Builder
	reader := bufio.NewReader(resp.Body)
	for {
		line, err := reader.ReadString('\n')
		if err != nil {
			if err == io.EOF {
				fmt.Println()
			} else {
				return contentBuilder.String(), err
			}
			break
		}
		contentBuilder.WriteString(line)
		fmt.Println(line)
	}
	return contentBuilder.String(), nil
}

type TestResult struct {
	Files        []OSSFile `json:"files"`
	SubmissionId string    `json:"submission_id"`
	Result       string    `json:"result"`
}

func SendAnalytics(requestBody *TestResult) {
	content, _ := json.Marshal(requestBody)
	_, _ = http.Post(REPORT_ENDPOINT, "application/json", bytes.NewReader(content))
}
